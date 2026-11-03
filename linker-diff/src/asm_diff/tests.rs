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

#[test]
fn reference_failure_keys_do_not_depend_on_diagnostic_text() {
    let r_type = X86RelocationType::from_raw(object::elf::R_X86_64_TLSGD);
    for message in [
        "Missing dynamic relocation for TlsGd",
        "Unable to read GOT at 0x1234",
    ] {
        let candidate = matched(object::elf::R_X86_64_TLSGD);
        let resolution = ResolvedGroup {
            relaxations: Some(RelaxationGroup {
                match_results: vec![RelaxationMatchResult::Matched(candidate)],
                is_complete: true,
            }),
            failure: Some(ResolutionFailure::Reference(r_type)),
            annotations: vec![
                candidate.annotation(),
                Annotation {
                    offset_in_section: 4,
                    kind: AnnotationKind::Error(message.to_owned()),
                },
            ],
            reference: Reference::unknown(),
            start: 0,
            end: 4,
            trace: TraceOutput::default(),
        };
        let reference = ResolvedGroup {
            failure: None,
            annotations: vec![candidate.annotation()],
            relaxations: Some(RelaxationGroup {
                match_results: vec![RelaxationMatchResult::Matched(candidate)],
                is_complete: true,
            }),
            reference: Reference::unknown(),
            start: 0,
            end: 4,
            trace: TraceOutput::default(),
        };
        assert!(resolution.has_error());
        assert_eq!(
            diff_key_for_res_mismatch(
                &[resolution, reference],
                &[],
                BinAttributes {
                    output_kind: OutputKind::Executable,
                    relocatability: Relocatability::Relocatable,
                    link_type: LinkType::Dynamic,
                }
            ),
            "rel.reference_failed.R_X86_64_TLSGD"
        );
    }
}

#[test]
fn reference_failure_keys_do_not_depend_on_reference_order() {
    let r_type = X86RelocationType::from_raw(object::elf::R_X86_64_TLSGD);
    let candidate = matched(object::elf::R_X86_64_TLSGD);
    let original_annotations = [OriginalAnnotation {
        success: MatchedRelaxation {
            r_type,
            relaxation_kind: None,
        },
        reference: Reference::unknown(),
        trace: TraceOutput::default(),
        offset: 0,
    }];
    let bin_attributes = BinAttributes {
        output_kind: OutputKind::Executable,
        relocatability: Relocatability::Relocatable,
        link_type: LinkType::Dynamic,
    };
    let mut resolutions = [false, false, true].map(|failed| ResolvedGroup {
        relaxations: Some(RelaxationGroup {
            match_results: vec![RelaxationMatchResult::Matched(candidate)],
            is_complete: true,
        }),
        failure: failed.then_some(ResolutionFailure::Reference(r_type)),
        annotations: vec![candidate.annotation()],
        reference: if failed {
            Reference::unknown()
        } else {
            Reference::direct(Referent::Absolute(42))
        },
        start: 0,
        end: 4,
        trace: TraceOutput::default(),
    });

    assert!(resolutions[0].matches(&resolutions[1]));
    assert!(!resolutions[0].matches(&resolutions[2]));
    for _ in 0..2 {
        assert_eq!(
            diff_key_for_res_mismatch(&resolutions, &original_annotations, bin_attributes),
            "rel.reference_failed.R_X86_64_TLSGD"
        );
        resolutions.swap(1, 2);
    }

    resolutions[0].failure = Some(ResolutionFailure::Ambiguous);
    assert_eq!(
        diff_key_for_res_mismatch(&resolutions, &original_annotations, bin_attributes),
        "rel.multiple_matches"
    );
}
