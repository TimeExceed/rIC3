use crate::options::*;

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.engine)?;
        match self.engine {
            Engine::BMC => {
                write!(f, "{}", self.bmc)?;
            }
            Engine::IC3 => {
                write!(f, "{}", self.ic3)?;
            }
            Engine::Kind => {
                write!(f, "{}", self.kind)?;
            }
            Engine::Portfolio => (),
        }
        if self.step != 1 {
            write!(f, " --step {}", self.step)?;
        }
        if self.rseed != 0 {
            write!(f, " --rseed {}", self.rseed)?;
        }
        write!(f, "{}", self.preprocess)?;
        Ok(())
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BMC => write!(f, "-e bmc"),
            Self::IC3 => write!(f, "-e ic3"),
            Self::Kind => write!(f, "-e kind"),
            Self::Portfolio => write!(f, "-e portfolio"),
        }
    }
}

impl std::fmt::Display for KindOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.no_bmc {
            write!(f, " --kind-no-bmc")?;
        }
        if self.kind_kissat {
            write!(f, " --kind-kissat")?;
        }
        if self.simple_path {
            write!(f, " --kind-simple-path")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for PreprocessOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sec {
            write!(f, " --sec")?;
        }
        if self.no_abc {
            write!(f, " --no-abc")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for BMCOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(x) = self.time_limit {
            write!(f, " --bmc-time-limit {}", x)?;
        }
        if self.bmc_kissat {
            write!(f, " --bmc-kissat")?;
        }
        if self.bmc_max_k < usize::MAX  {
            write!(f, " --bmc-max-k {}", self.bmc_max_k)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for IC3Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.dynamic {
            write!(f, " --ic3-dynamic")?;
        }
        if self.ctg {
            write!(f, " --ic3-ctg")?;
            if self.ctg_max != 3 {
                write!(f, " --ic3-ctg-max {}", self.ctg_max)?;
            }
            if self.ctg_limit != 1 {
                write!(f, " --ic3-ctg-limit {}", self.ctg_limit)?;
            }
        }
        if self.ctp {
            write!(f, " --ic3-ctp")?;
        }
        if self.inn {
            write!(f, " --ic3-inn")?;
        }
        if self.abs_cst {
            write!(f, " --ic3-abs-cst")?;
        }
        if self.no_pred_prop {
            write!(f, " --ic3-no-pred-prop")?;
        }
        Ok(())
    }
}
