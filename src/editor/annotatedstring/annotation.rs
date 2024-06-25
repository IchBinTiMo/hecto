use super::AnnotationType;

#[derive(Copy, Clone, Debug)]
#[allow(clippy::struct_field_name)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub start_byte_idx: usize,
    pub end_byte_idx: usize,
}
