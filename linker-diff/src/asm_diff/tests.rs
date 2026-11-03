use super::*;
use crate::x86_64::RType as X86RelocationType;
use crate::x86_64::X86_64;
use linker_utils::x86_64::RelaxationKind as X86RelaxationKind;

fn matched(r_type: object::elf::RelocationType) -> RelaxationMatch<'static, X86_64> {
    RelaxationMatch {
        relaxation: Relaxation {
            relaxation_kind: X86RelaxationKind::NoOp,
            new_r_type: X86RelocationType::from_raw(r_type),
            alt_r_type: None,
        },
        value: 0,
        start: 0,
        end: 4,
        original_referent: Referent::Unknown,
        addend: 0,
        offset: 0,
        next_modifier: RelocationModifier::Normal,
    }
}

#[test]
fn failure_without_candidate_annotations_keeps_its_input_relocation_type() {
    let group = RelaxationGroup::<X86_64> {
        match_results: vec![
            RelaxationMatchResult::Matched(matched(object::elf::R_X86_64_PC32)),
            RelaxationMatchResult::AllFailed(vec![]),
        ],
        is_complete: true,
    };
    let r_types = [object::elf::R_X86_64_GOTPCRELX, object::elf::R_X86_64_PLT32]
        .map(X86RelocationType::from_raw);

    assert!(group.match_results[1].annotations().is_empty());
    assert_eq!(
        group.failure(r_types.into_iter()).unwrap().diff_key(),
        "rel.match_failed.R_X86_64_PLT32"
    );
}

#[test]
fn failure_with_no_candidates_is_not_lost_before_an_ambiguous_match() {
    let candidate = matched(object::elf::R_X86_64_PC32);
    let group = RelaxationGroup::<X86_64> {
        match_results: vec![
            RelaxationMatchResult::AllFailed(vec![]),
            RelaxationMatchResult::Ambiguous(vec![candidate, candidate]),
        ],
        is_complete: true,
    };
    let r_types = [object::elf::R_X86_64_PLT32, object::elf::R_X86_64_GOTPCRELX]
        .map(X86RelocationType::from_raw);

    assert_eq!(
        group.failure(r_types.into_iter()).unwrap().diff_key(),
        "rel.match_failed.R_X86_64_PLT32"
    );
}
