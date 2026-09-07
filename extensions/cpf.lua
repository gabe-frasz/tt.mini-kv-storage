return {
	pre_hook = function(cmd, args)
		print("CPF pre hook - " .. cmd .. " " .. #args)
	end,

	post_hook = function(cmd, args)
		print("CPF post hook - " .. cmd .. " " .. #args)
	end,
}
