NUM_OF_INS = 1
NUM_OF_OUTS = 1
MEMORY_SIZE = 1

WIDTH = 2
HEIGHT = 2

INPUT_POSITIONS = {7}
OUTPUT_POSITIONS = {3}

function Calculate(inputs)
    local result = memory[1]
    memory[1] = inputs[1]
    return {result}
end

function Draw(buffer)
    buffer:set_all(0, 255, 255, 255)
end
