NUM_OF_INS = 1
NUM_OF_OUTS = 1
MEMORY_SIZE = 0

WIDTH = 1
HEIGHT = 3


INPUT_POSITIONS = {5}
OUTPUT_POSITIONS = {2}

function Calculate(inputs)
    return {not inputs[1]}
end

function Draw(buffer)
    buffer:set_all(255, 0, 0, 255)
end
