use crate::options;
use aig::Aig;
use giputils::hash::GHashMap;
use logic_form::Var;

pub fn aig_preprocess(aig: &Aig, _: &options::Options) -> (Aig, GHashMap<Var, Var>) {
    let (mut aig, remap) = aig.coi_refine();
    aig.constraints.retain(|e| !e.is_constant(true));
    (aig, remap)
}
