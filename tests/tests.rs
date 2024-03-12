#[cfg(test)]
mod tests {
    use new_logic_gates::Gate;
    use new_logic_gates::CalcMode;
    use new_logic_gates::LuaCode;

    #[test]
    fn test_calculate_and() -> mlua::Result<()> {
        let mut gate = Gate::new(
            "AND".to_string(),
            vec![true, true],
            vec![false],
        );

        // Lua-Code, der die calculate-Funktion definiert
        let lua_code = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = true
            for i, input in ipairs(inputs) do
                result = result and input
            end
            return {result}
        end
        "#;

        let lua_code = LuaCode(lua_code.to_string());

        // Führe die calculate-Funktion aus
        gate.calculate(&CalcMode::Lua(lua_code))?;

        // Überprüfe das Ergebnis
        assert_eq!(gate.get_outputs(), vec![true]);

        Ok(())
    }

    #[test]
    fn test_calculate_or() -> mlua::Result<()> {
        let mut gate = Gate::new(
            "OR".to_string(),
            vec![false, true],
            vec![false],
        );

        // Lua-Code, der die calculate-Funktion definiert
        let lua_code = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = false
            for i, input in ipairs(inputs) do
                result = result or input
            end
            return {result}
        end
        "#;

        let lua_code = &LuaCode(lua_code.to_string());

        // Führe die calculate-Funktion aus
        gate.calculate(&CalcMode::Lua(lua_code.clone()))?;

        // Überprüfe das Ergebnis
        assert_eq!(gate.get_outputs(), vec![true]);

        Ok(())
    }

    #[test]
    fn test_calculate_not() -> mlua::Result<()> {
        let mut gate = Gate::new(
            "NOT".to_string(),
            vec![true],
            vec![false],
        );

        // Lua-Code, der die calculate-Funktion definiert
        let lua_code: &str = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = not inputs[1]
            return {result}
        end
        "#;
        
        let lua_code = LuaCode(lua_code.to_string());
        
        // Führe die calculate-Funktion aus
        gate.calculate(&CalcMode::Lua(lua_code))?;
        
        // Überprüfe das Ergebnis
        assert_eq!(gate.get_outputs(), vec![false]);

        Ok(())
    }

    #[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use new_logic_gates::{BasicGate, Circuit, CircuitBus, Connection, LogicGate, TruthTable};

    use super::*;

    #[test]
    fn test_connection2() -> Result<(), Box<dyn Error>> {
        // Erstelle drei Gates
        let gate1 = Gate::new(
            "AND".to_string(),
            vec![true, false],
            vec![false],
        );
        let gate2 = Gate::new(
            "NOT".to_string(),
            vec![false],
            vec![true],
        );
        let gate3 = Gate::new(
            "OR".to_string(),
            vec![false, false],
            vec![false],
        );

        // Definiere die calculate-Funktionen für die Gates
        let lua_code1 = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = inputs[1] and inputs[2]
            return {result}
        end
        "#;
        let lua_code2  = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = not inputs[1]
            return {result}
        end
        "#;
        let lua_code3 = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = inputs[1] or inputs[2]
            return {result}
        end
        "#;

        let gate1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate1,
            CalcMode::Lua(LuaCode(lua_code1.to_string())),
        ))));
        
        let gate2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate2,
            CalcMode::Lua(LuaCode(lua_code2.to_string())),
        ))));

        let gate3: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate3,
            CalcMode::Lua(LuaCode(lua_code3.to_string())),
        ))));

        // Erstelle Verbindungen zwischen den Gates
        let mut connection1 = Connection::new(
            Rc::clone(&gate1),
            0,
            Rc::clone(&gate2),
            0,
        );
        let mut connection2 = Connection::new(
            Rc::clone(&gate2),
            0,
            Rc::clone(&gate3),
            0,
        );

        // Aktualisiere die Verbindungen
        gate1.borrow_mut().calculate()?;
        connection1.update();
        gate2.borrow_mut().calculate()?;
        connection2.update();
        gate3.borrow_mut().calculate()?;

        // Überprüfe das Ergebnis
        assert_eq!(gate3.borrow().get_inputs()[0], true);

        Ok(())
    }

    #[test]
    fn test_connection_with_changed_inputs() -> Result<(), Box<dyn Error>> {
        // Create three gates
        let gate1 = Gate::new("AND".to_string(), vec![true, false], vec![false]);
        let gate2 = Gate::new("NOT".to_string(), vec![false], vec![true]);
        let gate3 = Gate::new("OR".to_string(), vec![false, false], vec![false]);
    
        // Define the calculation functions for the gates
        let lua_code1 = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = inputs[1] and inputs[2]
            return {result}
        end
        "#;
    
        let lua_code2 = r#"
        NUM_OF_INS = 1
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = not inputs[1]
            return {result}
        end
        "#;
    
        let lua_code3 = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = inputs[1] or inputs[2]
            return {result}
        end
        "#;
    
        // Wrap the gates in Rc<RefCell<Box<dyn LogicGate>>> with their calculation modes
        let gate1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate1,
            CalcMode::Lua(LuaCode(lua_code1.to_string())),
        ))));
        let gate2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate2,
            CalcMode::Lua(LuaCode(lua_code2.to_string())),
        ))));
        let gate3: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate3,
            CalcMode::Lua(LuaCode(lua_code3.to_string())),
        ))));
    
        // Create connections between the gates
        let mut connection1 = Connection::new(Rc::clone(&gate1), 0, Rc::clone(&gate2), 0);
        let mut connection2 = Connection::new(Rc::clone(&gate2), 0, Rc::clone(&gate3), 0);
    
        // Perform the calculation functions
        gate1.borrow_mut().calculate()?;
        connection1.update();
        gate2.borrow_mut().calculate()?;
        connection2.update();
        gate3.borrow_mut().calculate()?;
    
        // Check the result
        assert_eq!(gate3.borrow().get_outputs()[0], true);
    
        // Change the inputs
        gate1.borrow_mut().set_input(0, true);
        gate1.borrow_mut().set_input(1, true);
    
        // Perform the calculation functions
        gate1.borrow_mut().calculate()?;
        connection1.update();
        gate2.borrow_mut().calculate()?;
        connection2.update();
        gate3.borrow_mut().calculate()?;
        
        // Re-check the result
        assert_eq!(gate2.borrow().get_outputs()[0], false);
        assert_eq!(gate3.borrow().get_inputs()[0], false);
        assert_eq!(gate3.borrow().get_inputs()[1], false);
        assert_eq!(gate3.borrow().get_outputs()[0], false);
    
        Ok(())
    }
    

    #[test]
    fn test_gate_memory() -> mlua::Result<()> {
        let mut gate = Gate::with_buffer(
            "BUFFER".to_string(),
            vec![true],
            vec![false],
            vec![false],
        );

        // Lua-Code, der die calculate-Funktion definiert
        let buffer_code = std::fs::read_to_string("./comps/buffer.lua")?;

        let lua_code = LuaCode(buffer_code.to_string());

        // Führe die calculate-Funktion aus
        gate.calculate(&CalcMode::Lua(lua_code.clone()))?;
        assert_eq!(gate.get_outputs(), vec![false]);
        gate.calculate(&CalcMode::Lua(lua_code.clone()))?;
        assert_eq!(gate.get_outputs(), vec![true]);
        
        gate.set_input(0, false);
        gate.calculate(&CalcMode::Lua(lua_code.clone()))?;
        assert_eq!(gate.get_outputs(), vec![true]);
        gate.calculate(&CalcMode::Lua(lua_code))?;
        assert_eq!(gate.get_outputs(), vec![false]);

        Ok(())
    }

    #[test]
    fn test_gate_to_tt() {
        let mut gate = Gate::new(
            "AND".to_string(),
            vec![true, true],
            vec![false],
        );

        let lua_code = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1

        function Calculate(inputs)
            local result = inputs[1] and inputs[2]
            return {result}
        end
        "#;

        let lua_code = LuaCode(lua_code.to_string());

        gate.calculate(&CalcMode::Lua(lua_code.clone())).unwrap();

        let tt = new_logic_gates::compile_gate_to_truth_table(&mut gate, &lua_code).unwrap();

        println!("Truth Table: {:?}", tt.map);

        fn verify_truth_table(tt: &TruthTable) -> bool {
            let expected_outputs = vec![
                (vec![false, false], vec![false]),
                (vec![false, true], vec![false]),
                (vec![true, false], vec![false]),
                (vec![true, true], vec![true]),
            ];
        
            for (inputs, expected_output) in expected_outputs {
                let output = tt.map.get(&inputs).unwrap(); // Assuming this is how you get outputs for inputs from your TruthTable
                if output[0] != expected_output[0] {
                    return false; // The truth table does not match the expected output for these inputs
                }
            }
        
            true // All entries matched the expected outputs
        }

        let is_correct = verify_truth_table(&tt);
        assert!(is_correct, "The truth table does not represent an AND gate correctly.");
    }

    #[test]
    fn test_circuit() -> Result<(), Box<dyn Error>> {
        let mut circuit = Circuit::new("Test Circuit".to_string());

        // Create three gates
        let gate1 = Gate::new("AND".to_string(), vec![true, false], vec![false]);
        let gate2 = Gate::new("NOT".to_string(), vec![false], vec![true]);
        let gate3 = Gate::new("OR".to_string(), vec![false, false], vec![false]);
    
        // Define the calculation functions for the gates
        let lua_code1 = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = inputs[1] and inputs[2]
            return {result}
        end
        "#;
    
        let lua_code2 = r#"
        NUM_OF_INS = 1
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = not inputs[1]
            return {result}
        end
        "#;
    
        let lua_code3 = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = inputs[1] or inputs[2]
            return {result}
        end
        "#;
    
        // Wrap the gates in Rc<RefCell<Box<dyn LogicGate>>> with their calculation modes
        let and_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate1,
            CalcMode::Lua(LuaCode(lua_code1.to_string())),
        ))));
        let not_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate2,
            CalcMode::Lua(LuaCode(lua_code2.to_string())),
        ))));
        let or_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            gate3,
            CalcMode::Lua(LuaCode(lua_code3.to_string())),
        ))));

        let input1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let input2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let output1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));

        circuit.add_input(input1.clone());
        circuit.add_input(input2.clone());
        circuit.add_output(output1.clone());

        circuit.add_gate(and_gate.clone());
        circuit.add_gate(not_gate.clone());
        circuit.add_gate(or_gate.clone());

        circuit.conn_input_to_gate(0, and_gate.clone(), 0).unwrap();
        circuit.conn_input_to_gate(0, or_gate.clone(), 1).unwrap();
        circuit.conn_input_to_gate(1, and_gate.clone(), 1).unwrap();

        circuit.connect(and_gate, 0, or_gate.clone(), 0);
        circuit.connect(or_gate, 0, not_gate.clone(), 0);

        circuit.conn_gate_to_output(0, not_gate, 0)?;

        circuit.set_input(0, false);
        circuit.set_input(1, false);

        circuit.calculate()?;

        assert_eq!(circuit.get_outputs()[0], true);

        Ok(())
    }    


    #[test]
    fn test_recursive_circuit() -> Result<(), Box<dyn Error>> {
        let mut circuit = Circuit::new("NAND".to_string());

        // Create three gates
        let and = Gate::new("AND".to_string(), vec![true, false], vec![false]);
        let not = Gate::new("NOT".to_string(), vec![false], vec![true]);
    
        // Define the calculation functions for the gates
        let and_code = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = inputs[1] and inputs[2]
            return {result}
        end
        "#;
    
        let not_code = r#"
        NUM_OF_INS = 1
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = not inputs[1]
            return {result}
        end
        "#;
    
        // Wrap the gates in Rc<RefCell<Box<dyn LogicGate>>> with their calculation modes
        let and_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            and,
            CalcMode::Lua(LuaCode(and_code.to_string())),
        ))));
        let not_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            not,
            CalcMode::Lua(LuaCode(not_code.to_string())),
        ))));

        let input1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let input2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let output1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));

        circuit.add_input(input1.clone());
        circuit.add_input(input2.clone());
        circuit.add_output(output1.clone());

        circuit.add_gate(and_gate.clone());
        circuit.add_gate(not_gate.clone());

        circuit.conn_input_to_gate(0, and_gate.clone(), 0).unwrap();
        circuit.conn_input_to_gate(1, and_gate.clone(), 1).unwrap();

        circuit.connect(and_gate, 0, not_gate.clone(), 0);

        circuit.conn_gate_to_output(0, not_gate, 0)?;

        circuit.set_input(0, true);
        circuit.set_input(1, false);
        circuit.calculate()?;
        assert_eq!(circuit.get_outputs()[0], true);

        let mut circuit2 = Circuit::new("Test".to_string());
        let input1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let input2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let output2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));

        let circuit: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(circuit)));

        circuit2.add_input(input1.clone());
        circuit2.add_input(input2.clone());
        circuit2.add_output(output2.clone());
        circuit2.add_gate(circuit.clone());

        circuit2.conn_input_to_gate(0, circuit.clone(), 0).unwrap();
        circuit2.conn_input_to_gate(1, circuit.clone(), 1).unwrap();
        circuit2.conn_gate_to_output(0, circuit.clone(), 0)?;

        circuit2.set_input(0, true);
        circuit2.set_input(1, true);

        circuit2.calculate()?;
        assert_eq!(circuit2.get_outputs()[0], false);

        Ok(())
    }

    #[test]
    fn test_compile_circuit() -> Result<(), Box<dyn Error>> {
        let mut circuit = Circuit::new("NAND".to_string());

        // Create three gates
        let and = Gate::new("AND".to_string(), vec![true, false], vec![false]);
        let not = Gate::new("NOT".to_string(), vec![false], vec![true]);
    
        // Define the calculation functions for the gates
        let and_code = r#"
        NUM_OF_INS = 2
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = inputs[1] and inputs[2]
            return {result}
        end
        "#;
    
        let not_code = r#"
        NUM_OF_INS = 1
        NUM_OF_OUTS = 1
    
        function Calculate(inputs)
            local result = not inputs[1]
            return {result}
        end
        "#;
    
        // Wrap the gates in Rc<RefCell<Box<dyn LogicGate>>> with their calculation modes
        let and_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            and,
            CalcMode::Lua(LuaCode(and_code.to_string())),
        ))));
        let not_gate: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(BasicGate::from_gate(
            not,
            CalcMode::Lua(LuaCode(not_code.to_string())),
        ))));

        let input1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let input2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let output1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));

        circuit.add_input(input1.clone());
        circuit.add_input(input2.clone());
        circuit.add_output(output1.clone());

        circuit.add_gate(and_gate.clone());
        circuit.add_gate(not_gate.clone());

        circuit.conn_input_to_gate(0, and_gate.clone(), 0).unwrap();
        circuit.conn_input_to_gate(1, and_gate.clone(), 1).unwrap();

        circuit.connect(and_gate, 0, not_gate.clone(), 0);

        circuit.conn_gate_to_output(0, not_gate, 0)?;

        circuit.set_input(0, true);
        circuit.set_input(1, false);
        circuit.calculate()?;
        assert_eq!(circuit.get_outputs()[0], true);

        let mut tt = TruthTable::new();
        tt.add(vec![false, false], vec![true]);
        tt.add(vec![false, true], vec![true]);
        tt.add(vec![true, false], vec![true]);
        tt.add(vec![true, true], vec![false]);
        let comp_tt = circuit.compile().unwrap();

        assert_eq!(tt.map, comp_tt.map);

        let mut circuit2 = Circuit::new("Test".to_string());
        let input1: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let input2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));
        let output2: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(CircuitBus::new())));

        let circuit: Rc<RefCell<Box<dyn LogicGate>>> = Rc::new(RefCell::new(Box::new(circuit)));

        circuit2.add_input(input1.clone());
        circuit2.add_input(input2.clone());
        circuit2.add_output(output2.clone());
        circuit2.add_gate(circuit.clone());

        circuit2.conn_input_to_gate(0, circuit.clone(), 0).unwrap();
        circuit2.conn_input_to_gate(1, circuit.clone(), 1).unwrap();
        circuit2.conn_gate_to_output(0, circuit.clone(), 0)?;

        circuit2.set_input(0, true);
        circuit2.set_input(1, true);

        circuit2.calculate()?;
        assert_eq!(circuit2.get_outputs()[0], false);

        Ok(())
    }


}
}


