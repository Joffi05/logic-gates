NUM_OF_INS = 1
NUM_OF_OUTS = 1
MEMORY_SIZE = 0

WIDTH = 2
HEIGHT = 1


INPUT_POSITIONS = {1}
OUTPUT_POSITIONS = {1}

function Calculate(inputs)
    return {not inputs[1]}
end

function Draw(buffer)
    buffer:set_all(255, 0, 0, 255)
end
