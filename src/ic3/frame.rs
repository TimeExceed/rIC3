use super::{IC3, proofoblig::ProofObligation};
use crate::transys::TransysCtx;
use aig::*;
use giputils::grc::Grc;
use giputils::hash::{GHashSet, GHashMap};
use logic_form::{Lemma, Lit, LitSet, LitVec, Var};
use satif::Satif;
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct FrameLemma {
    lemma: Lemma,
    pub po: Option<ProofObligation>,
    pub _ctp: Option<LitVec>,
}

impl FrameLemma {
    #[inline]
    pub fn new(lemma: Lemma, po: Option<ProofObligation>, ctp: Option<LitVec>) -> Self {
        Self {
            lemma,
            po,
            _ctp: ctp,
        }
    }
}

impl Deref for FrameLemma {
    type Target = Lemma;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lemma
    }
}

impl DerefMut for FrameLemma {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lemma
    }
}

impl FrameLemma {
    pub(crate) fn display<'a>(&'a self, aig: &'a Aig) -> FrameLemmaDisplay<'a> {
        FrameLemmaDisplay { frame_lemma: self, aig }
    }
}

pub(crate) struct FrameLemmaDisplay<'a> {
    frame_lemma: &'a FrameLemma,
    aig: &'a Aig,
}

impl std::fmt::Display for FrameLemmaDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", LemmaDisplay {
            lemma: &self.frame_lemma.lemma,
            aig: self.aig,
        })
    }
}

pub(crate) struct LemmaDisplay<'a> {
    pub(crate) lemma: &'a Lemma,
    pub(crate) aig: &'a Aig,
}

impl std::fmt::Display for LemmaDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for lit in self.lemma.iter() {
            writeln!(f)?;
            write!(f, "    {}", LitDisplay { lit, aig: self.aig })?;
        }
        Ok(())
    }
}

pub(crate) struct LitVecDispaly<'a> {
    pub(crate) lits: &'a [Lit],
    pub(crate) aig: &'a Aig,
}

impl std::fmt::Display for LitVecDispaly<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for lit in self.lits.iter() {
            writeln!(f)?;
            write!(f, "    {}", LitDisplay { lit, aig: self.aig })?;
        }
        Ok(())
    }
}

pub struct LitDisplay<'a> {
    pub lit: &'a Lit,
    pub aig: &'a Aig,
}

impl std::fmt::Display for LitDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_lit(f, self.aig, *self.lit)?;
        Ok(())
    }
}

fn fmt_lit(
    f: &mut std::fmt::Formatter<'_>,
    aig: &Aig,
    lit: Lit,
) -> std::fmt::Result {
    // let var = lit.var();
    // if let Some(n) = aig.symbols.get(&(var.0 as usize)) {
    //     write!(f, "{n}")?;
    // } else {
    //     write!(f, "{{{}}}", var.0)?;
    // }
    if lit.polarity() {
        fmt_(f, aig, !lit)?;
        write!(f, " <- ⊤")?;
    } else {
        fmt_(f, aig, lit)?;
        write!(f, " <- ⊥")?;
    }
    Ok(())
}

fn fmt_(
    f: &mut std::fmt::Formatter<'_>,
    aig: &Aig,
    lit: Lit,
) -> std::fmt::Result {
    if lit.polarity() {
        write!(f, "!(")?;
        fmt_(f, aig, !lit)?;
        write!(f, ")")?;
        return Ok(());
    }
    let var: usize = lit.var().into();
    if let Some(n) = aig.symbols.get(&var) {
        write!(f, "{n}")?;
        return Ok(());
    }
    if var < aig.latchs.len() + aig.inputs.len() {
        write!(f, "{{{}}}", var)?;
        return Ok(());
    }
    fmt_aig_node(f, aig, var)?;
    Ok(())
}

fn fmt_aig_node(
    f: &mut std::fmt::Formatter<'_>,
    aig:&Aig,
    idx: usize,
) -> std::fmt::Result {
    let node = &aig.nodes[idx];
    match &node.typ {
        AigNodeType::False => {
            return write!(f, "⊥");
        }
        AigNodeType::Leaf => {
            if let Some(n) = aig.symbols.get(&idx) {
                return write!(f, "{n}");
            }
            assert!(idx < aig.latchs.len() + aig.inputs.len());
            return write!(f, "{{{idx}}}");
        }
        AigNodeType::And(lhs, rhs) => {
            write!(f, "(")?;
            fmt_aig_edge(f, aig, lhs)?;
            write!(f, ") ∧ (")?;
            fmt_aig_edge(f, aig, rhs)?;
            write!(f, ")")?;
            return Ok(());
        }
    }
}

fn fmt_aig_edge(
    f: &mut std::fmt::Formatter<'_>,
    aig:&Aig,
    edge: &AigEdge,
) -> std::fmt::Result {
    if edge.complement {
        write!(f, "¬")?;
    }
    fmt_aig_node(f, aig, edge.id)?;
    Ok(())
}

pub struct Frame {
    lemmas: Vec<FrameLemma>,
}

impl Frame {
    pub fn new() -> Self {
        Self { lemmas: Vec::new() }
    }
}

impl Deref for Frame {
    type Target = Vec<FrameLemma>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lemmas
    }
}

impl DerefMut for Frame {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lemmas
    }
}

impl Frame {
    pub(crate) fn display<'a>(&'a self, aig: &'a Aig) -> FrameDisplay<'a> {
        FrameDisplay { frame: self, aig }
    }
}

pub(crate) struct FrameDisplay<'a> {
    frame: &'a Frame,
    aig: &'a Aig,
}

impl std::fmt::Display for FrameDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, lemma) in self.frame.lemmas.iter().enumerate() {
            write!(f, "  lemma {i}: {}", lemma.display(self.aig))?;
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct Frames {
    frames: Vec<Frame>,
    pub early: usize,
    pub tmp_lit_set: LitSet,
}

impl Frames {
    pub fn new(ts: &Grc<TransysCtx>) -> Self {
        let mut tmp_lit_set = LitSet::new();
        tmp_lit_set.reserve(ts.max_latch);
        Self {
            frames: Default::default(),
            early: 1,
            tmp_lit_set,
        }
    }

    #[inline]
    pub fn trivial_contained<'a>(
        &'a mut self,
        frame: usize,
        lemma: &Lemma,
    ) -> Option<(usize, &'a mut Option<ProofObligation>)> {
        for l in lemma.iter() {
            self.tmp_lit_set.insert(*l);
        }
        for (i, fi) in self.frames.iter_mut().enumerate().skip(frame) {
            for j in 0..fi.len() {
                if fi[j].lemma.subsume_set(lemma, &self.tmp_lit_set) {
                    self.tmp_lit_set.clear();
                    return Some((i, &mut fi[j].po));
                }
            }
        }
        self.tmp_lit_set.clear();
        None
    }

    pub fn invariant(&self) -> Vec<Lemma> {
        let invariant = self.iter().position(|frame| frame.is_empty()).unwrap();
        let mut invariants = Vec::new();
        for i in invariant..self.len() {
            for cube in self[i].iter() {
                invariants.push(cube.deref().clone());
            }
        }
        invariants.sort();
        invariants
    }

    pub fn _parent_lemma(&self, lemma: &Lemma, frame: usize) -> Option<Lemma> {
        if frame == 1 {
            return None;
        }
        for c in self.frames[frame - 1].iter() {
            if c.subsume(lemma) {
                return Some(c.lemma.clone());
            }
        }
        None
    }

    pub fn _parent_lemmas(&self, lemma: &Lemma, frame: usize) -> Vec<Lemma> {
        let mut res = Vec::new();
        if frame == 1 {
            return res;
        }
        for c in self.frames[frame - 1].iter() {
            if c.subsume(lemma) {
                res.push(c.lemma.clone());
            }
        }
        res
    }

    #[allow(unused)]
    pub fn similar(&self, cube: &[Lit], frame: usize) -> Vec<LitVec> {
        let cube_set: GHashSet<Lit> = GHashSet::from_iter(cube.iter().copied());
        let mut res = GHashSet::new();
        for frame in self.frames[frame..].iter() {
            for lemma in frame.iter() {
                let sec: LitVec = lemma
                    .iter()
                    .filter(|l| cube_set.contains(l))
                    .copied()
                    .collect();
                if sec.len() != cube.len() && sec.len() * 2 >= cube.len() {
                    res.insert(sec);
                }
            }
        }
        let mut res = Vec::from_iter(res);
        res.sort_by_key(|x| x.len());
        res.reverse();
        if res.len() > 3 {
            res.truncate(3);
        }
        res
    }

    #[inline]
    pub fn statistic(&self) {
    }
}

impl Deref for Frames {
    type Target = Vec<Frame>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl DerefMut for Frames {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl Frames {
    #[inline]
    pub fn get_mut(&mut self) -> &mut Vec<Frame> {
        &mut self.frames
    }

    pub(crate) fn display<'a>(&'a self, aig: &'a Aig) -> FramesDisplay<'a> {
        FramesDisplay { frames: self, aig }
    }
}

pub(crate) struct FramesDisplay<'a> {
    frames: &'a Frames,
    aig: &'a Aig,
}

impl std::fmt::Display for FramesDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "inspect into frames.")?;
        for (i, frame) in self.frames.iter().enumerate() {
            writeln!(f, "frame {i}: {} lemmas", frame.lemmas.len())?;
            write!(f, "{}", frame.display(self.aig))?;
        }
        Ok(())
    }
}

impl IC3 {
    #[inline]
    pub fn add_lemma(
        &mut self,
        frame: usize,
        lemma: LitVec,
        contained_check: bool,
        po: Option<ProofObligation>,
    ) -> bool {
        let lemma = Lemma::new(lemma);
        if frame == 0 {
            assert!(self.frame.len() == 1);
            self.solvers[0].add_lemma(&!lemma.cube());
            if !self.options.ic3.no_pred_prop && self.level() == frame {
                self.bad_solver.add_clause(&!lemma.cube());
            }
            self.frame[0].push(FrameLemma::new(lemma, po, None));
            return false;
        }
        if contained_check && self.frame.trivial_contained(frame, &lemma).is_some() {
            return false;
        }
        if self.ts.cube_subsume_init(lemma.cube()) {
            assert!(self.options.ic3.inn);
        }
        let mut begin = None;
        let mut inv_found = false;
        'fl: for i in (1..=frame).rev() {
            let mut j = 0;
            while j < self.frame[i].len() {
                let l = &self.frame[i][j];
                if begin.is_none() && l.subsume(&lemma) {
                    if l.eq(&lemma) {
                        self.frame[i].swap_remove(j);
                        let clause = !lemma.cube();
                        for k in i + 1..=frame {
                            self.solvers[k].add_lemma(&clause);
                        }
                        if !self.options.ic3.no_pred_prop && self.level() == frame {
                            self.bad_solver.add_clause(&!lemma.cube());
                        }
                        self.frame[frame].push(FrameLemma::new(lemma, po, None));
                        self.frame.early = self.frame.early.min(i + 1);
                        return self.frame[i].is_empty();
                    } else {
                        begin = Some(i + 1);
                        break 'fl;
                    }
                }
                if lemma.subsume(l) {
                    let _remove = self.frame[i].swap_remove(j);
                    // self.solvers[i].remove_lemma(&remove);
                    continue;
                }
                j += 1;
            }
            if i != frame && self.frame[i].is_empty() {
                inv_found = true;
            }
        }
        let clause = !lemma.cube();
        let begin = begin.unwrap_or(1);
        for i in begin..=frame {
            self.solvers[i].add_lemma(&clause);
        }
        if !self.options.ic3.no_pred_prop && self.level() == frame {
            self.bad_solver.add_clause(&!lemma.cube());
        }
        self.frame[frame].push(FrameLemma::new(lemma, po, None));
        self.frame.early = self.frame.early.min(begin);
        inv_found
    }

    // pub fn remove_lemma(&mut self, frame: usize, lemmas: Vec<LitVec>) {
    //     let lemmas: GHashSet<Lemma> = GHashSet::from_iter(lemmas.into_iter().map(Lemma::new));
    //     for i in (1..=frame).rev() {
    //         let mut j = 0;
    //         while j < self.frame[i].len() {
    //             if let Some(po) = &mut self.frame[i][j].po {
    //                 po.removed = true;
    //             }
    //             if lemmas.contains(&self.frame[i][j]) {
    //                 for s in self.solvers[..=frame].iter_mut() {
    //                     s.remove_lemma(&self.frame[i][j]);
    //                 }
    //                 self.frame[i].swap_remove(j);
    //             } else {
    //                 j += 1;
    //             }
    //         }
    //     }
    // }
}
