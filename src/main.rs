#![feature(ptr_metadata)]

use aig::Aig;
use clap::Parser;
use rIC3::{
    Engine,
    bmc::BMC,
    frontend::aig::aig_preprocess,
    ic3::IC3,
    kind::Kind,
    options::{self, Options},
    portfolio::portfolio_main,
    transys::Transys,
};
use jiff::Timestamp;
use giputils::hash::GHashMap;
use logic_form::*;

use std::{
    fs,
    mem::{self, transmute},
    process::exit,
    ptr,
};

fn main() {
    logforth::builder()
        .dispatch(|d| {
            use logforth::{filter::EnvFilter, append::Stderr};
            d
                .filter(EnvFilter::from_default_env_or("info"))
                .append(Stderr::default().with_layout(MyLayout{}))
        })
        .apply();

    fs::create_dir_all("/tmp/rIC3").unwrap();
    let mut options = Options::parse();
    options.model = options.model.canonicalize().unwrap();
    SESSION_NAME.set(format!("{options}")).unwrap();
    let res = if let options::Engine::Portfolio = options.engine {
        portfolio_main(options)
    } else {
        raw_main(options)
    };
    if let Some(res) = res {
        exit(if res { 20 } else { 10 })
    } else {
        exit(0)
    }
}

fn raw_main(options: Options) -> Option<bool> {
    let mut aig = match options.model.extension() {
        Some(ext) if (ext == "btor") | (ext == "btor2") => panic!(
            "rIC3 currently does not support parsing BTOR2 files. Please use btor2aiger (https://github.com/hwmcc/btor2tools) to first convert them to AIG format."
        ),
        Some(ext) if (ext == "aig") | (ext == "aag") => {
            Aig::from_file(options.model.to_str().unwrap())
        }
        _ => panic!("unsupported file format"),
    };
    if !aig.outputs.is_empty() && !options.certify {
        // not certifying, move outputs to bads
        // Move outputs to bads if no bad properties exist
        if aig.bads.is_empty() {
            aig.bads = std::mem::take(&mut aig.outputs);
            println!(
                "Warning: property not found, moved {} outputs to bad properties",
                aig.bads.len()
            );
        } else {
            println!("Warning: outputs are ignored");
        }
    }

    let origin_aig = aig.clone();
    assert!(options.index < aig.bads.len(),
        "--index {}, # of bads: {}", options.index, aig.bads.len());
    let old_aig = &aig;
    let (mut new_aig, restore) = aig_preprocess(&old_aig, &options);
    update_symbols(&mut new_aig, old_aig, &restore);
    let ts = Transys::from_aig(&new_aig, &restore);
    if options.preprocess.sec {
        panic!("sec not support");
    }
    let mut engine: Box<dyn Engine> = match options.engine {
        options::Engine::IC3 => Box::new(IC3::new(options.clone(), ts, new_aig, vec![])),
        options::Engine::Kind => Box::new(Kind::new(options.clone(), ts)),
        options::Engine::BMC => Box::new(BMC::new(options.clone(), ts)),
        _ => unreachable!(),
    };
    if options.interrupt_statistic {
        let e: (usize, usize) =
            unsafe { transmute((engine.as_mut() as *mut dyn Engine).to_raw_parts()) };
        let _ = ctrlc::set_handler(move || {
            let e: *mut dyn Engine = unsafe {
                ptr::from_raw_parts_mut(
                    e.0 as *mut (),
                    transmute::<usize, std::ptr::DynMetadata<dyn rIC3::Engine>>(e.1),
                )
            };
            let e = unsafe { &mut *e };
            e.statistic();
            exit(124);
        });
    }
    let res = engine.check();
    engine.statistic();
    match res {
        Some(true) => {
            if !options.json_output {
                println!("result: safe");
            }
            // certificate(&mut engine, &new_aig, &options, true);
        }
        Some(false) => {
            if options.json_output {
                // println!("{}", engine.witness(&origin_aig));
                println!("[]");
            } else {
                println!("result: unsafe");
                // println!("{}", engine.witness(&origin_aig));
            }
            // certificate(&mut engine, &new_aig, &options, false);
        }
        _ => {
            println!("result: unknown");
        }
    }
    mem::forget(engine);
    res
}

#[derive(Debug)]
struct MyLayout {}

impl logforth::Layout for MyLayout {
    fn format(&self, record: &log::Record, _: &[logforth::Diagnostic]) -> anyhow::Result<Vec<u8>> {
        let tm = Timestamp::now();
        let session = SESSION_NAME.get().unwrap();
        let message = record.args();
        let res = format!(
            "{tm:.3} \"{session}\": [{}:{}] {message}",
            if let Some(f) = record.file() {
                f
            } else {
                ""
            },
            if let Some(l) = record.line() {
                l
            } else {
                0
            },
        );

        Ok(res.into_bytes())
    }
}

static SESSION_NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn update_symbols(new_aig: &mut Aig, old_aig: &Aig, var_map: &GHashMap<Var, Var>) {
    let mut new_symbs = {
        let raw_symbs = &old_aig.symbols;
        let mut res = GHashMap::new();
        for (new_var, old_var) in var_map.iter() {
            let old_var: usize = (*old_var).into();
            if let Some(n) = raw_symbs.get(&old_var) {
                res.insert(new_var.0 as usize, n.clone());
            }
        }
        res
    };
    std::mem::swap(&mut new_symbs, &mut new_aig.symbols);
}
