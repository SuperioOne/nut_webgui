#!/bin/lua
require("io")

-- Reads dev paths from stdin and generates macro inputs for integration tests.

---@param input string
---@return string
local function normalize(input)
	return input:gsub("[-.$~(){}'+]", "_")
end

for path in io.stdin:lines() do
	for manufacturer, model, driver, nut_ver, report_no in path:gmatch(".*/(.+)__(.+)__(.+)__(.+)__(.+).dev$") do
		local test_name = string.format(
			"DEV_%s__%s__%s__v%s__r%s",
			normalize(manufacturer):upper(),
			normalize(model):upper(),
			normalize(driver),
			normalize(nut_ver),
			normalize(report_no)
		)

		print(string.format("(%-90s,%q);", test_name, path))
	end
end
