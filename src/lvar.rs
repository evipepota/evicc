use crate::sema::Type;

#[derive(Clone, Debug)]
pub struct LVar {
    pub next: Option<Box<LVar>>,
    pub name: String,
    pub offset: i32,
    pub ty: Type,
}

impl LVar {
    pub fn new(next: Option<Box<LVar>>, name: String, offset: i32, ty: Type) -> Self {
        LVar {
            next,
            name,
            offset,
            ty,
        }
    }
}
