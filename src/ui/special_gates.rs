use crate::LogicGate;


pub struct ButtonGate {
    pub state: bool,
}

impl ButtonGate {
    pub fn new() -> Self {
        Self {
            state: false,
        }
    }
}

impl LogicGate for ButtonGate {
    fn get_name(&self) -> String {
        "BUTTON".to_string()
    }

    fn get_inputs(&self) -> Vec<bool> {
        vec![]
    }

    fn get_outputs(&self) -> Vec<bool> {
        vec![self.state]
    }

    // Does nothing for Button gate
    fn set_input(&mut self, _index: usize, _value: bool) { }

    fn set_output(&mut self, _index: usize, value: bool) {
        self.state = value;
    }

    fn calculate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn compilable(&self) -> bool {
        false
    }

    fn compile(&mut self) -> Result<crate::TruthTable, crate::CantCompileGate> {
        Err(crate::CantCompileGate)
    }
    
    fn get_input_num(&self) -> usize {
        0
    }
    
    fn get_output_num(&self) -> usize {
        1
    }
}

pub struct LampGate {
    pub state: bool,
}

impl LampGate {
    pub fn new() -> Self {
        Self {
            state: false,
        }
    }
}

impl LogicGate for LampGate {
    fn get_name(&self) -> String {
        "LAMP".to_string()
    }

    fn get_inputs(&self) -> Vec<bool> {
        vec![self.state]
    }

    fn get_outputs(&self) -> Vec<bool> {
        vec![]
    }

    fn set_input(&mut self, _index: usize, value: bool) {
        self.state = value;
    }

    // Does nothing for Lamp gate
    fn set_output(&mut self, _index: usize, _value: bool) { }

    fn calculate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn compilable(&self) -> bool {
        false
    }

    fn compile(&mut self) -> Result<crate::TruthTable, crate::CantCompileGate> {
        Err(crate::CantCompileGate)
    }
    
    fn get_input_num(&self) -> usize {
        1
    }
    
    fn get_output_num(&self) -> usize {
        0
    }
}