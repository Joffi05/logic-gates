use core::fmt;
use std::error::Error;
use std::{cell::RefCell, vec};
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use mlua::{
    Function, Lua,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod ui;


#[derive(Debug, Clone)]
pub struct TruthTable{
    pub map: HashMap<Vec<bool>, Vec<bool>>
}

impl TruthTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    pub fn add(&mut self, inputs: Vec<bool>, outputs: Vec<bool>) {
        self.map.insert(inputs, outputs);
    }

    pub fn get(&self, inputs: Vec<bool>) -> Vec<bool> {
        self.map.get(&inputs).unwrap().clone()
    }
}

// Helper function to convert Vec<bool> to a string representation.
fn vec_bool_to_string(vec: &Vec<bool>) -> String {
    vec.iter().map(|&b| if b { '1' } else { '0' }).collect()
}
fn string_to_vec_bool(s: &str) -> Vec<bool> {
    s.chars().map(|c| c == '1').collect()
}


impl Serialize for TruthTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the HashMap into a Vec of (String, Vec<bool>) pairs
        let map_as_vec: Vec<(String, &Vec<bool>)> = self
            .map
            .iter()
            .map(|(key, value)| (vec_bool_to_string(key), value))
            .collect();
        
        // Serialize this Vec as a map
        map_as_vec.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for TruthTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a Vec of (String, Vec<bool>) pairs, which was our intermediate format
        let vec: Vec<(String, Vec<bool>)> = Vec::deserialize(deserializer)?;

        // Convert this Vec back into the original HashMap format
        let map = vec.into_iter().map(|(key, value)| (string_to_vec_bool(&key), value)).collect();

        Ok(TruthTable { map })
    }
}

#[derive(Debug)]
pub struct CantConnect {
    err: String,
}

impl Error for CantConnect {}

impl fmt::Display for CantConnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't connect gates because: {}", self.err)
    }
}

pub struct CircuitBus {
    mem: bool,
}

impl CircuitBus {
    pub fn new() -> Self {
        Self {
            mem: false,
        }
    }

}

impl LogicGate for CircuitBus {
    fn get_name(&self) -> String {
        "BUS".to_string()
    }

    fn get_inputs(&self) -> Vec<bool> {
        vec![self.mem]
    }

    fn get_outputs(&self) -> Vec<bool> {
        vec![self.mem]
    }

    fn set_input(&mut self, _index: usize, value: bool) {
        self.mem = value;
    }

    fn set_output(&mut self, _index: usize, value: bool) {
        self.mem = value;
    }

    fn calculate(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn compilable(&self) -> bool {
        true
    }

    fn compile(&mut self) -> Result<TruthTable, CantCompileGate> {
        let mut tt = TruthTable::new();
        tt.add(vec![false], vec![false]);
        tt.add(vec![true], vec![true]);
        Ok(tt)
    }
}


// Structure that holds many gates and can be compiled to a new gate
pub struct Circuit {
    name: String,
    gates: Vec<Rc<RefCell<Box<dyn LogicGate>>>>,
    connections: Vec<Connection>,
    circuit_inputs: Vec<Rc<RefCell<Box<dyn LogicGate>>>>,
    circuit_outputs: Vec<Rc<RefCell<Box<dyn LogicGate>>>>,
}

impl Circuit {
    pub fn new(name: String) -> Self {
        Self {
            name,
            gates: Vec::new(),
            connections: Vec::new(),
            circuit_inputs: Vec::new(),
            circuit_outputs: Vec::new(),
        }
    }

    pub fn add_input(&mut self, input: Rc<RefCell<Box<dyn LogicGate>>>) -> Rc<RefCell<Box<dyn LogicGate>>> {
        self.circuit_inputs.push(input.clone());
        input
    }

    pub fn add_output(&mut self, output: Rc<RefCell<Box<dyn LogicGate>>>) -> Rc<RefCell<Box<dyn LogicGate>>> {
        self.circuit_outputs.push(output.clone());
        output
    }

    pub fn add_gate(&mut self, gate: Rc<RefCell<Box<dyn LogicGate>>>) -> Rc<RefCell<Box<dyn LogicGate>>> {
        self.gates.push(gate.clone());
        gate
    }

    pub fn conn_input_to_gate(&mut self, input_num: usize, gate: Rc<RefCell<Box<dyn LogicGate>>>, dest_in_num: usize) -> Result<(), CantConnect> {
        // Return error if "input_num" is out of range
        if input_num >= self.circuit_inputs.len() {
            return Err(CantConnect { err: "Input number out of range".to_string() });
        }
    
        // Check if "gate" is in self.gates by comparing Rc pointers
        let gate_in_circuit = self.gates.iter().any(|g| Rc::ptr_eq(g, &gate));
    
        if !gate_in_circuit {
            return Err(CantConnect { err: "Gate not in circuit".to_string() });
        }
    
        let conn = Connection::new(self.circuit_inputs[input_num].clone(), 0, gate, dest_in_num);
        self.connections.push(conn);

        Ok(())
    }

    pub fn conn_gate_to_output(&mut self, output_num: usize, gate: Rc<RefCell<Box<dyn LogicGate>>>, src_out_num: usize) -> Result<(), CantConnect> {
        // Return error if "output_num" is out of range
        if output_num >= self.circuit_outputs.len() {
            return Err(CantConnect { err: format!("Output number: {} out of range", output_num).to_string() });
        }
    
        // Check if "gate" is in self.gates by comparing Rc pointers
        let gate_in_circuit = self.gates.iter().any(|g| Rc::ptr_eq(g, &gate));
    
        if !gate_in_circuit {
            return Err(CantConnect { err: format!("Gate {} not in circuit", gate.borrow().get_name()).to_string() });
        }

        //Check if any gates are already connected to the output
        for conn in &self.connections {
            if Rc::ptr_eq(&conn.get_output_gate(), &self.circuit_outputs[output_num]) {
                return Err(CantConnect { err: format!("Gate {} already connected to output {}", gate.borrow().get_name(), output_num).to_string() });
            }
        }
    
        let conn = Connection::new(gate, src_out_num, self.circuit_outputs[output_num].clone(), 0);
        self.connections.push(conn);

        Ok(())
    }
    
    pub fn compilable(&self) -> bool {
        for gate in self.gates.iter() {
            if !gate.borrow().compilable() {
                return false;
            }
        }
        true
    }

    pub fn connect(&mut self, src_gate: Rc<RefCell<Box<dyn LogicGate>>>, src_index: usize, dest_gate: Rc<RefCell<Box<dyn LogicGate>>>, dest_index: usize) {
        let connection = Connection::new(src_gate, src_index, dest_gate, dest_index);
        self.connections.push(connection);
    }
}

impl LogicGate for Circuit {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_inputs(&self) -> Vec<bool> {
        self.circuit_inputs.iter().map(|gate| gate.borrow().get_inputs()[0]).collect()
    }

    fn get_outputs(&self) -> Vec<bool> {
        self.circuit_outputs.iter().map(|gate| gate.borrow().get_outputs()[0]).collect()
    }

    fn set_input(&mut self, index: usize, value: bool) {
        self.circuit_inputs[index].borrow_mut().set_input(0, value);
    }

    fn set_output(&mut self, index: usize, value: bool) {
        self.circuit_outputs[index].borrow_mut().set_output(0, value);
    }

    fn calculate(&mut self) -> Result<(), Box<dyn Error>> {
        // Set of gates already updated in this cycle to avoid infinite loops
        let mut updated_gates = HashSet::new();

        // Start with input gates
        let mut to_update: Vec<Rc<RefCell<Box<dyn LogicGate>>>> = self.circuit_inputs.clone();
        let mut to_update_next: Vec<Rc<RefCell<Box<dyn LogicGate>>>>;

        while !to_update.is_empty() {
            to_update_next = Vec::new();

            for gate_rc in to_update.iter() {
                // Skip if this gate has already been updated
                if updated_gates.contains(&(Rc::as_ptr(gate_rc) as *const _)) {
                    continue;
                }

                // Calculate based on current inputs, which may internally update the gate's state
                gate_rc.borrow_mut().calculate()?;
                updated_gates.insert(Rc::as_ptr(gate_rc) as *const _);

                // For each connection where this gate is the source, add the destination gate to the update list
                for conn in self.connections.iter_mut() {
                    conn.update();
                    if Rc::ptr_eq(&conn.get_input_gate(), gate_rc) {
                        to_update_next.push(conn.get_output_gate().clone());
                    }
                }
            }

            to_update = to_update_next;
        }

        Ok(())
    }

    fn compilable(&self) -> bool {
        self.compilable()
    }

    //TODO hier müsste eig &self ausreichen aber weil ich keine copy erstellen kann muss och das orig nehmen
    fn compile(&mut self) -> Result<TruthTable, CantCompileGate> {
        if !self.compilable() {
            return Err(CantCompileGate);
        }

        let mut table = TruthTable::new();

        for i in 0..2_usize.pow(self.circuit_inputs.len() as u32) {
            let binary = format!("{:0width$b}", i, width = self.get_inputs().len());
            let inputs: Vec<bool> = binary.chars().map(|c| c == '1').collect();
            // Change all input gates to the current input
            for (i, input) in self.circuit_inputs.iter_mut().enumerate() {
                input.borrow_mut().set_input(0, inputs[i]);
            }
            if let Err(_) = self.calculate() {
                return Err(CantCompileGate);
            }
            let outputs = self.get_outputs();
            table.add(inputs, outputs);
        }

        Ok(table)
    }
}

pub fn compile_gate_to_truth_table(gate: &mut Gate, code: &LuaCode) -> Result<TruthTable, CantCompileGate> {
    let mut table = TruthTable::new();

    if gate.is_stateful() {
        return Err(CantCompileGate);
    }

    // Iterate over all possible input combinations
    for i in 0..2_usize.pow(gate.inputs.len() as u32) {
        let binary = format!("{:0width$b}", i, width = gate.inputs.len());
        let inputs: Vec<bool> = binary.chars().map(|c| c == '1').collect();
        gate.inputs = inputs.clone();  // Make sure the gate's inputs are updated for each iteration
        gate.calculate(&CalcMode::Lua(code.clone())).unwrap();
        let outputs = gate.outputs.clone();
        table.add(inputs, outputs);
    }

    Ok(table)
}

pub struct Gate {
    name: String,
    inputs: Vec<bool>,
    outputs: Vec<bool>,
    memory: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct LuaCode(pub String);
pub enum CalcMode {
    Lua(LuaCode),
    TruthTable(TruthTable),
}

impl Gate {
    pub fn new(name: String, inputs: Vec<bool>, outputs: Vec<bool>) -> Self {
        Self {
            name,
            inputs,
            outputs,
            memory: vec![],
        }
    }

    pub fn with_buffer(name: String, inputs: Vec<bool>, outputs: Vec<bool>, memory: Vec<bool>) -> Self {
        Self {
            name,
            inputs,
            outputs,
            memory,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_inputs(&self) -> Vec<bool> {
        self.inputs.clone().to_vec()
    }

    pub fn get_outputs(&self) -> Vec<bool> {
        self.outputs.clone().to_vec()
    }

    pub fn set_input(&mut self, index: usize, value: bool) {
        self.inputs[index] = value;
    }

    pub fn set_output(&mut self, index: usize, value: bool) {
        self.outputs[index] = value;
    }

    pub fn is_stateful(&self) -> bool {
        if self.memory.len() != 0 {
            return true;
        }
        
        false
    }

    pub fn calculate(&mut self, calc: &CalcMode) -> mlua::Result<()> {
        match calc {
            CalcMode::Lua(lua_code) => {
                let lua = Lua::new();

                let globals = lua.globals();
                
                // Lade den lua code
                lua.load(&lua_code.0).exec()?;

                // Check for needed variables
                globals.get::<_, u8>("NUM_OF_INS").expect("NUM_OF_INS is needed in every gate");
                globals.get::<_, u8>("NUM_OF_OUTS").expect("NUM_OF_OUTS is needed in every gate");

                if self.memory.len() != 0 {
                    globals.set("memory", self.memory.clone())?;
                }

                let calculate: Function = globals.get("Calculate")?;
                let output = calculate.call::<_, Vec<bool>>(self.inputs.clone())?;
                self.outputs = output;

                if self.memory.len() != 0 {
                    self.memory = globals.get::<_, Vec<bool>>("memory")?;
                }
            },
            CalcMode::TruthTable(_table) => {
                todo!("TruthTable")
            },
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CantCompileGate;

impl Error for CantCompileGate {}

impl fmt::Display for CantCompileGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't compile gate to truth table")
    }
}

pub fn save_truth_table(table: &TruthTable, path: &str) -> Result<(), Box<dyn Error>> {
    // Serialize the truth table to a JSON string
    let json = serde_json::to_string(table)?;
    
    // Write the JSON string to the specified file path
    std::fs::write(path, json)?;
    
    Ok(())
}
pub fn load_truth_table(path: &str) -> Result<TruthTable, Box<dyn Error>> {
    // Read the contents of the file as a JSON string
    let json = std::fs::read_to_string(path)?;
    
    // Deserialize the JSON string into a TruthTable object
    let table: TruthTable = serde_json::from_str(&json)?;
    
    Ok(table)
}

pub struct BasicGate {
    gate: Gate,
    calc_mode: CalcMode,
}

impl BasicGate {
    pub fn from_gate(gate: Gate, calc_mode: CalcMode) -> Self {
        Self {
            gate,
            calc_mode,
        }
    }
}

pub fn new_and() -> Result<BasicGate, Box<dyn Error>> {
    let gate = Gate::new("AND".to_string(), vec![false, false], vec![false]);
    let tt = load_truth_table("./comps/and.json")?; 
    let calc_mode = CalcMode::TruthTable(tt);
    Ok(BasicGate::from_gate(gate, calc_mode))
}
pub fn new_or() -> Result<BasicGate, Box<dyn Error>> {
    let gate = Gate::new("OR".to_string(), vec![false, false], vec![false]);
    let tt = load_truth_table("./comps/or.json")?; 
    let calc_mode = CalcMode::TruthTable(tt);
    Ok(BasicGate::from_gate(gate, calc_mode))
}
pub fn new_not() -> Result<BasicGate, Box<dyn Error>> {
    let gate = Gate::new("NOT".to_string(), vec![false], vec![false]);
    let tt = load_truth_table("./comps/not.json")?; 
    let calc_mode = CalcMode::TruthTable(tt);
    Ok(BasicGate::from_gate(gate, calc_mode))
}
pub fn new_buffer() -> Result<BasicGate, Box<dyn Error>> {
    let gate = Gate::with_buffer("BUFFER".to_string(), vec![false], vec![false], vec![false]);
    let code = std::fs::read("./comps/buffer.lua")?;
    let code = LuaCode(String::from_utf8(code).unwrap());
    let calc_mode = CalcMode::Lua(code);
    Ok(BasicGate::from_gate(gate, calc_mode))
}

pub trait LogicGate {
    fn get_name(&self) -> String;
    fn get_inputs(&self) -> Vec<bool>;
    fn get_outputs(&self) -> Vec<bool>;
    fn set_input(&mut self, index: usize, value: bool);
    fn set_output(&mut self, index: usize, value: bool);
    fn calculate(&mut self) -> Result<(), Box<dyn Error>>;
    fn compilable(&self) -> bool;
    fn compile(&mut self) -> Result<TruthTable, CantCompileGate>;
}

impl LogicGate for BasicGate {
    fn get_name(&self) -> String {
        self.gate.get_name()
    }

    fn get_inputs(&self) -> Vec<bool> {
        self.gate.get_inputs()
    }

    fn get_outputs(&self) -> Vec<bool> {
        self.gate.get_outputs()
    }

    fn set_input(&mut self, index: usize, value: bool) {
        self.gate.set_input(index, value)
    }

    fn set_output(&mut self, index: usize, value: bool) {
        self.gate.set_output(index, value)
    }

    fn calculate(&mut self) -> Result<(), Box<dyn Error>> {
        self.gate.calculate(&self.calc_mode)?;
        Ok(())
    }

    fn compilable(&self) -> bool {
        !self.gate.is_stateful()
    }

    fn compile(&mut self) -> Result<TruthTable, CantCompileGate> {
        if self.compilable() {
            match &self.calc_mode {
                CalcMode::Lua(code) => {
                    compile_gate_to_truth_table(&mut self.gate, &code)
                },
                CalcMode::TruthTable(tt) => {
                    Ok(tt.clone())
                }
            }
        }
        else {
            Err(CantCompileGate)
        
        }
    }
}


pub struct Connection {
    src_gate: Rc<RefCell<Box<dyn LogicGate>>>,
    src_index: usize,
    dest_gate: Rc<RefCell<Box<dyn LogicGate>>>,
    dest_index: usize,
}

impl Connection {
    pub fn new(
        src_gate: Rc<RefCell<Box<dyn LogicGate>>>,
        src_index: usize,
        dest_gate: Rc<RefCell<Box<dyn LogicGate>>>,
        dest_index: usize,
    ) -> Self {
        Self {
            src_gate,
            src_index, 
            dest_gate,
            dest_index,
        }
    }

    pub fn get_input_gate(&self) -> Rc<RefCell<Box<dyn LogicGate>>> {
        self.src_gate.clone()
    }

    pub fn get_output_gate(&self) -> Rc<RefCell<Box<dyn LogicGate>>> {
        self.dest_gate.clone()
    }

    pub fn get_input_index(&self) -> usize {
        self.src_index
    }

    pub fn get_output_index(&self) -> usize {
        self.dest_index
    }

    pub fn update(&mut self) {
        let input = self.src_gate.borrow().get_outputs()[self.src_index];
        self.dest_gate.borrow_mut().set_input(self.dest_index, input);
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut and_gate = Gate::new("AND".to_string(), vec![false, false], vec![false]);
    let mut or_gate = Gate::new("OR".to_string(), vec![false, false], vec![false]);
    let mut not_gate = Gate::new("NOT".to_string(), vec![false], vec![false]);


    // Load the lua and code
    let code = std::fs::read("./comps/and.lua").unwrap();
    let code = LuaCode(String::from_utf8(code).unwrap());
    let tt = compile_gate_to_truth_table(&mut and_gate, &code).unwrap();
    save_truth_table(&tt, "./comps/and.json").unwrap();

    let code = std::fs::read("./comps/or.lua").unwrap();
    let code = LuaCode(String::from_utf8(code).unwrap());
    let tt = compile_gate_to_truth_table(&mut or_gate, &code).unwrap();
    save_truth_table(&tt, "./comps/or.json").unwrap();

    let code = std::fs::read("./comps/not.lua").unwrap();
    let code = LuaCode(String::from_utf8(code).unwrap());
    let tt = compile_gate_to_truth_table(&mut not_gate, &code).unwrap();
    save_truth_table(&tt, "./comps/not.json").unwrap();
    
    Ok(())
}