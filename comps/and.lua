NUM_OF_INS = 2
NUM_OF_OUTS = 1
MEMORY_SIZE = 0

WIDTH = 3
HEIGHT = 2

function Calculate(inputs)
    local result = true
    for i, input in ipairs(inputs) do
        result = result and input
    end
    return {result}
end

function Draw(buffer)
    buffer:set_all(255, 0, 0, 255)
end
