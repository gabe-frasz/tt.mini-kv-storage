return {
	prefix = "cpf_",

	pre_hook = function(ctx)
		if ctx.command == "ADD" then
			local clean = ctx.value:gsub("%D", "")
			if #clean ~= 11 then
				error("CPF must have 11 digits")
			end
			ctx.value = clean
		end
	end,

	post_hook = function(ctx)
		if ctx.command == "GET" and ctx.result then
			ctx.result = ctx.result:gsub("(%d%d%d)(%d%d%d)(%d%d%d)(%d%d)", "%1.%2.%3-%4")
		end
	end,
}
