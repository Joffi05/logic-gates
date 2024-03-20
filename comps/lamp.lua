NUM_OF_INS = 1
NUM_OF_OUTS = 0
MEMORY_SIZE = 1

WIDTH = 2
HEIGHT = 2

INPUT_POSITIONS = {7}
OUTPUT_POSITIONS = {}

function Calculate(inputs)
    return {inputs[1]}
end

function Draw(buffer)
    buffer:set_all(0, 255, 255, 255)
end
