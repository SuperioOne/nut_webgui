require("io")

-- Reads variable list from stdin and generates macro inputs for standard variable names.

---@param input string
---@return string
local function into_variant_name(input)
	local words = {}
	local i = 1

	for part in input:gmatch("%w+") do
		local first = part:sub(1, 1)
		local slice = part:sub(2)
		words[i] = string.format("%s%s", first:upper(), slice)
		i = i + 1
	end

	local result = ""
	for _, part in pairs(words) do
		result = result .. part
	end

	return result
end

for line in io.stdin:lines("l") do
	local variant_name = into_variant_name(line)
	local const_name = line:upper():gsub("%.", "_")

	print(string.format("(%-52s,%-48s,%q);", const_name, variant_name, line))
end
