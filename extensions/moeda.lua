local function parse_and_validate(val)
	if val:find(",") then
		error("formato monetário inválido: utilize ponto como separador decimal")
	end

	if not val:match("^%d+%.?%d*$") then
		error("valor monetário inválido: deve conter apenas dígitos e ponto decimal opcional")
	end

	local int_part, dec_part = val:match("^(%d+)%.?(%d*)$")
	if not int_part then
		error("valor monetário inválido")
	end

	if #dec_part > 2 then
		error("valor monetário inválido: máximo de duas casas decimais permitidas")
	end

	-- Pad decimal part to 2 digits: "" -> "00", "5" -> "50", "50" -> "50"
	if #dec_part == 0 then
		dec_part = "00"
	elseif #dec_part == 1 then
		dec_part = dec_part .. "0"
	end

	-- Remove leading zeros from integer part, unless it's just "0"
	int_part = int_part:gsub("^0+(%d)", "%1")

	local cents_str = int_part .. dec_part
	-- Strip leading zeros from total cents: "005" -> "5", "000" -> "0"
	cents_str = cents_str:gsub("^0+(%d)", "%1")

	return cents_str
end

local function format_currency(cents_str)
	local cents_num = tonumber(cents_str) or 0
	local reais = math.floor(cents_num / 100)
	local centavos = cents_num % 100

	-- Format thousands separator with dots: 1234567 -> 1.234.567
	local s = tostring(reais)
	local formatted_reais = ""
	while #s > 3 do
		formatted_reais = "." .. s:sub(-3) .. formatted_reais
		s = s:sub(1, -4)
	end
	formatted_reais = s .. formatted_reais

	return string.format("R$ %s,%02d", formatted_reais, centavos)
end

return {
	prefix = "moeda_",

	pre_hook = function(ctx)
		if ctx.command == "ADD" then
			local cents = parse_and_validate(ctx.value)
			ctx.value = cents
		end
	end,

	post_hook = function(ctx)
		if ctx.command == "GET" and ctx.result then
			ctx.result = format_currency(ctx.result)
		end
	end,
}
