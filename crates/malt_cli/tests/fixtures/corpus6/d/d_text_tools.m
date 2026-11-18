function out = d_text_tools(input, mode)
%R2_TEXT_TOOLS Clean and normalise short text fields.
%   MODE is 'slug', 'title' or 'digits'. Strings may be char or string.

if nargin < 2
    mode = 'slug';
end

if isstring(input)
    input = char(input);
end

switch mode
    case 'slug'
        out = lower(strtrim(input));
        out = regexprep(out, '[^a-z0-9]+', '-');
        out = regexprep(out, '^-|-$', '');
    case 'title'
        words = strsplit(strtrim(input), ' ');
        for i = 1:numel(words)
            w = words{i};
            if ~isempty(w)
                words{i} = [upper(w(1)) lower(w(2:end))];
            end
        end
        out = strjoin(words, ' ');
    case 'digits'
        out = input(isstrprop(input, 'digit'));
    otherwise
        error('Unknown mode: %s', mode);
end

label = "Result: " + out
tags = ["a", "b"] + "-" + string(numel(out))
s = 'it''s done';
d = "say ""hi""";
n = length(s) + strlength(d);

m = containers.Map();
m('x') = 1;
m('y') = 2;
keysList = keys(m);
for k = 1:numel(keysList)
    key = keysList{k};
    val = m(key);
end

parts = textscan('1,2,3', '%d', 'Delimiter', ',');
total = sum(parts{1});
fprintf('total=%d\n', total);
end
