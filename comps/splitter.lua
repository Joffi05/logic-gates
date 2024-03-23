NUM_OF_INS = 1
NUM_OF_OUTS = 0
MEMORY_SIZE = 0

HEIGHT = 0
WIDTH = 0

INPUT_POSITIONS = {0}
OUTPUT_POSITIONS = {}

function Calculate(inputs)
    return {inputs[1]}
end


-- Blue
function Draw(buffer)
    buffer:set_all(0,0,255, 255)
end
