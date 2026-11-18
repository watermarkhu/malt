function [value, pos] = recursive_parser(s, pos)
%RECURSIVE_PARSER Parses sums, products and parentheses of integers.
%   [value, pos] = RECURSIVE_PARSER('2+3*(4+1)', 1)

if nargin < 2
    pos = 1;
end
[value, pos] = parseSum(s, pos);
if pos <= numel(s)
    error('parser:trailing', 'unexpected text at %d', pos);
end

    function [v, p] = parseSum(str, p)
        [v, p] = parseProduct(str, p);
        while p <= numel(str) && any(str(p) == '+-')
            op = str(p);
            [rhs, p] = parseProduct(str, p + 1);
            if op == '+'
                v = v + rhs;
            else
                v = v - rhs;
            end
        end
    end

    function [v, p] = parseProduct(str, p)
        [v, p] = parseAtom(str, p);
        while p <= numel(str) && str(p) == '*'
            [rhs, p] = parseAtom(str, p + 1);
            v = v * rhs;
        end
    end

    function [v, p] = parseAtom(str, p)
        if str(p) == '('
            [v, p] = parseSum(str, p + 1);
            if p > numel(str) || str(p) ~= ')'
                error('parser:paren', 'missing )');
            end
            p = p + 1;
        else
            q = p;
            while q <= numel(str) && isstrprop(str(q), 'digit')
                q = q + 1;
            end
            if q == p
                error('parser:digit', 'digit expected at %d', p);
            end
            v = str2double(str(p:q-1));
            p = q;
        end
    end
end
