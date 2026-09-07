RAW_MATCH = "%d%d%d%d%-%d%d%-%d%d"
GROUP_MATCH = "(%d%d%d%d)%-(%d%d)%-(%d%d)"
OUPUT_FORMAT = "%3/%2/%1"
DAYS_IN_MONTH = { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 }

return {
  prefix = "data_",

  pre_hook = function(ctx)
    if ctx.command == "ADD" then
      if not ctx.value:match(RAW_MATCH) then
        error("Data must be on format YYYY-MM-DD")
      end

      local year, month, day = ctx.value:match(GROUP_MATCH)
      year, month, day = tonumber(year), tonumber(month), tonumber(day)

      local is_leap_year = year % 4 == 0 and (year % 100 ~= 0 or year % 400 == 0)
      DAYS_IN_MONTH[2] = is_leap_year and 29 or 28

      if month < 1 or month > 12 then
        error("Invalid month")
      end

      if day < 1 or day > DAYS_IN_MONTH[month] then
        error("Invalid day for month")
      end
    end
  end,

  post_hook = function(ctx)
    if ctx.command == "GET" and ctx.result then
      ctx.result = ctx.result:gsub(GROUP_MATCH, OUPUT_FORMAT)
    end
  end,
}
