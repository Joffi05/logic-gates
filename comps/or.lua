NUM_OF_INS = 2
NUM_OF_OUTS = 1
MEMORY_SIZE = 0

HEIGHT = 2
WIDTH = 3

INPUT_POSITIONS = {1, 3}
OUTPUT_POSITIONS = {2}

function Calculate(inputs)
    local result = true
    for i, input in ipairs(inputs) do
        result = result or input
    end
    return {result}
end


-- Blue
function Draw(buffer)
    buffer:set_all(0,0,255, 255)
end
