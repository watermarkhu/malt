function toks = string_tokens(txt)
% STRING_TOKENS Splits a small expression language into tokens.
%   Numbers, identifiers, operators and parentheses. Whitespace is dropped.

toks = {};
i = 1;
n = length(txt);
while i <= n
    ch = txt(i);
    if isspace(ch)
        i = i + 1;
        continue
    elseif isletter(ch)
        j = i;
        while j <= n && (isletter(txt(j)) || isdigit(txt(j)))
            j = j + 1;
        end
        toks{end+1} = struct('type', 'id', 'text', txt(i:j-1));
        i = j;
    elseif isdigit(ch) || ch == '.'
        [val, count] = sscanf(txt(i:end), '%f', 1);
        if isempty(count) || count == 0
            error('tokens:number', 'bad number at %d', i);
        end
        toks{end+1} = struct('type', 'num', 'text', num2str(val));
        i = i + max(1, floor(log10(abs(val)+1)) + 1);
    elseif any(ch == '+-*/^')
        toks{end+1} = struct('type', 'op', 'text', ch);
        i = i + 1;
    elseif ch == '(' || ch == ')'
        toks{end+1} = struct('type', 'paren', 'text', ch);
        i = i + 1;
    else
        error('tokens:char', 'unexpected character "%s"', ch);
    end
end
kinds = cellfun(@(t) t.type, toks, 'UniformOutput', false);
nIds = sum(strcmp(kinds, 'id'));
fprintf('%d tokens, %d identifiers\n', numel(toks), nIds);
end
