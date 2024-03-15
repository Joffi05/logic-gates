NUM_OF_INS = 2
NUM_OF_OUTS = 1
MEMORY_SIZE = 0

HEIGHT = 2
WIDTH = 3

function Calculate(inputs)
    local result = true
    for i, input in ipairs(inputs) do
        result = result or input
    end
    return {result}
end