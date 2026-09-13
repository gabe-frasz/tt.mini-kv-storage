-- extensions/json.lua
-- Extensão para manipulação e atualização atômica de documentos JSON planos
-- Prefixo: json_
-- Suporta tipos primitivos: string, number (inteiro/float) e boolean

local function trim(s)
    return s:match("^%s*(.-)%s*$")
end

local function skip_whitespace(s, pos)
    return s:find("%S", pos) or (#s + 1)
end

-- Decodifica um valor JSON primitivo a partir da posição `pos`
local function parse_value(s, pos)
    pos = skip_whitespace(s, pos)
    if pos > #s then
        return nil, "valor esperado não encontrado", pos
    end

    local ch = s:sub(pos, pos)

    -- String: "..."
    if ch == '"' then
        local start_str = pos + 1
        local end_str = pos + 1
        local escaped = false
        while end_str <= #s do
            local c = s:sub(end_str, end_str)
            if c == '\\' then
                escaped = not escaped
            elseif c == '"' and not escaped then
                break
            else
                escaped = false
            end
            end_str = end_str + 1
        end
        if end_str > #s then
            return nil, "string com aspas não fechadas", pos
        end
        local val = s:sub(start_str, end_str - 1)
        val = val:gsub('\\"', '"'):gsub('\\\\', '\\')
        return val, end_str + 1
    end

    -- Booleanos: true / false
    if s:sub(pos, pos + 3) == "true" then
        return true, pos + 4
    elseif s:sub(pos, pos + 4) == "false" then
        return false, pos + 5
    end

    -- Números: inteiros ou decimais (com sinal opcional)
    local num_str = s:match("^%-?%d+%.?%d*", pos)
    if num_str and num_str ~= "" and num_str ~= "-" then
        local n = tonumber(num_str)
        if n ~= nil then
            return n, pos + #num_str
        end
    end

    return nil, "tipo de valor inválido ou não suportado (permitidos: string entre aspas, number ou boolean)", pos
end

-- Faz o parsing de um objeto JSON plano: {"campo": valor, ...}
local function parse_json(s)
    local trimmed = trim(s)
    if not (trimmed:sub(1, 1) == "{" and trimmed:sub(-1) == "}") then
        return nil, "JSON deve iniciar com '{' e terminar com '}'"
    end

    local body = trim(trimmed:sub(2, -2))
    local result = {}
    if body == "" then
        return result
    end

    local pos = 1
    local len = #body

    while pos <= len do
        pos = skip_whitespace(body, pos)
        if pos > len then break end

        -- Chave deve começar com aspas duplas
        if body:sub(pos, pos) ~= '"' then
            return nil, "chave do JSON deve estar entre aspas duplas"
        end

        local key_end = body:find('"', pos + 1)
        if not key_end then
            return nil, "chave do JSON com aspas não fechadas"
        end

        local key = body:sub(pos + 1, key_end - 1)
        if key == "" then
            return nil, "chave do JSON não pode ser vazia"
        end
        pos = key_end + 1

        -- Esperado ':'
        pos = skip_whitespace(body, pos)
        if pos > len or body:sub(pos, pos) ~= ':' then
            return nil, "esperado ':' após o nome da chave"
        end
        pos = pos + 1

        -- Valor
        local val, next_pos = parse_value(body, pos)
        if val == nil then
            return nil, next_pos
        end
        result[key] = val
        pos = next_pos

        -- Esperado ',' ou fim do objeto
        pos = skip_whitespace(body, pos)
        if pos <= len then
            if body:sub(pos, pos) == ',' then
                pos = pos + 1
            else
                return nil, "esperado ',' ou '}' separando os campos"
            end
        end
    end

    return result
end

-- Serializa tabela Lua para JSON minificado determinístico (chaves ordenadas)
local function encode_json(tbl)
    local keys = {}
    for k in pairs(tbl) do
        table.insert(keys, k)
    end
    table.sort(keys)

    local parts = {}
    for _, k in ipairs(keys) do
        local v = tbl[k]
        local v_str
        if type(v) == "string" then
            v_str = string.format("%q", v)
        elseif type(v) == "number" then
            v_str = tostring(v)
        elseif type(v) == "boolean" then
            v_str = v and "true" or "false"
        end
        table.insert(parts, string.format("%q:%s", k, v_str))
    end
    return "{" .. table.concat(parts, ",") .. "}"
end

-- Formata tabela para visualização em tabela ASCII multilinha no GET (chaves ordenadas)
local function format_table(tbl)
    local keys = {}
    for k in pairs(tbl) do
        table.insert(keys, k)
    end
    table.sort(keys)

    if #keys == 0 then
        return "{}"
    end

    -- Calcula largura das colunas
    local max_k = 5 -- comprimento de "CAMPO"
    local max_v = 5 -- comprimento de "VALOR"
    for _, k in ipairs(keys) do
        if #k > max_k then max_k = #k end
        local v = tbl[k]
        local v_str = type(v) == "string" and string.format("%q", v) or tostring(v)
        if #v_str > max_v then max_v = #v_str end
    end

    local sep = "+" .. string.rep("-", max_k + 2) .. "+" .. string.rep("-", max_v + 2) .. "+"
    local lines = {
        sep,
        string.format("| %-" .. max_k .. "s | %-" .. max_v .. "s |", "CAMPO", "VALOR"),
        sep,
    }
    for _, k in ipairs(keys) do
        local v = tbl[k]
        local v_str = type(v) == "string" and string.format("%q", v) or tostring(v)
        table.insert(lines, string.format("| %-" .. max_k .. "s | %-" .. max_v .. "s |", k, v_str))
    end
    table.insert(lines, sep)
    return table.concat(lines, "\n")
end

-- Aplica patch parcial no objeto: @campo=val, +@campo=val ou -@campo
local function apply_patch(tbl, patch_cmd)
    local p = trim(patch_cmd)

    -- Remoção idempotente: -@campo (não gera erro se o campo não existir)
    local del_key = p:match("^%-@([%w_]+)$")
    if del_key then
        tbl[del_key] = nil
        return tbl
    end

    -- Atualização ou Inserção idempotente: @campo=valor ou +@campo=valor
    local field, val_raw = p:match("^[%+@]?@([%w_]+)%s*=%s*(.+)$")
    if not field then
        return nil, "sintaxe de atualização inválida (use @campo=valor, +@campo=valor ou -@campo)"
    end

    local val, err = parse_value(val_raw, 1)
    if val == nil then
        return nil, "valor inválido no patch: " .. tostring(err)
    end

    tbl[field] = val
    return tbl
end

return {
    prefix = "json_",

    pre_hook = function(ctx)
        if ctx.command == "ADD" then
            local raw = trim(ctx.value)

            if raw:sub(1, 1) == "{" then
                -- Modo 1: Inicialização completa ou sobrescrita do documento
                local tbl, err = parse_json(raw)
                if not tbl then
                    error(err)
                end
                ctx.value = encode_json(tbl)
            elseif raw:sub(1, 1) == "@" or raw:sub(1, 2) == "+@" or raw:sub(1, 2) == "-@" then
                -- Modo 2: Atualização parcial atômica idempotente (patch) via ctx.get(ctx.key)
                local current = ctx.get(ctx.key)
                local tbl = {}
                if current then
                    local parsed, err = parse_json(current)
                    if not parsed then
                        error("JSON existente corrompido: " .. tostring(err))
                    end
                    tbl = parsed
                end
                local patched, patch_err = apply_patch(tbl, raw)
                if not patched then
                    error(patch_err)
                end
                ctx.value = encode_json(patched)
            else
                error("valor deve ser um objeto JSON '{...}' ou um comando de atualização (@campo=valor, +@campo=valor, -@campo)")
            end
        end
    end,

    post_hook = function(ctx)
        if ctx.command == "GET" and ctx.result then
            local tbl, _ = parse_json(ctx.result)
            if tbl then
                ctx.result = format_table(tbl)
            end
        end
    end,
}
