#[derive(Debug)]
pub struct Script<'a> {
    pub id: &'a str,
    pub stack: Vec<Branch<'a>>,
}

#[derive(Debug)]
pub enum Branch<'a> {
    Repeat(RepeatBranch<'a>),
    Until(UntilBranch<'a>),
    Forever(ForeverBranch<'a>),
}

#[derive(Debug)]
pub struct RepeatBranch<'a> {
    pub iterations: u32,
    pub return_id: &'a str,
    pub branch_id: &'a str,
}

#[derive(Debug)]
pub struct UntilBranch<'a> {
    pub condition: &'a str,
    pub return_id: &'a str,
    pub branch_id: &'a str,
}

#[derive(Debug)]
pub struct ForeverBranch<'a> {
    pub branch_id: &'a str,
}
