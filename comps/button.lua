NUM_OF_INS = 0
NUM_OF_OUTS = 1
MEMORY_SIZE = 1

WIDTH = 2
HEIGHT = 2

INPUT_POSITIONS = {}
OUTPUT_POSITIONS = {3}

function Calculate(inputs)
    return {memory[1]}
end

function Draw(buffer)
    buffer:set_all(0, 255, 255, 255)
end
