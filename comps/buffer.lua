NUM_OF_INS = 1
NUM_OF_OUTS = 1
MEMORY_SIZE = 1

function Calculate(inputs)
    local result = memory[1]
    memory[1] = inputs[1]
    return {result}
end