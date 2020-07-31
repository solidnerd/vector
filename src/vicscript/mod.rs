use crate::event::Event;

pub mod parser;
pub mod query;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug)]
pub struct Assignment {
    path: String,
    function: Box<dyn query::Function>,
}

impl Assignment {
    pub fn new(path: String, func: Box<dyn query::Function>) -> Self {
        Self {
            path: path,
            function: func,
        }
    }

    fn apply(&self, target: &mut Event) -> Result<()> {
        let v = self.function.execute(&target)?;
        target.as_mut_log().insert(&self.path, v);
        Ok(())
    }
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Mapping {
    assignments: Vec<Assignment>,
}

impl Mapping {
    pub fn new(assignments: Vec<Assignment>) -> Self {
        Mapping { assignments }
    }

    pub fn execute(&self, event: &mut Event) -> Result<()> {
        for (i, assignment) in self.assignments.iter().enumerate() {
            if let Err(err) = assignment.apply(event) {
                return Err(format!("failed to apply mapping {}: {}", i, err));
            }
        }
        Ok(())
    }
}
