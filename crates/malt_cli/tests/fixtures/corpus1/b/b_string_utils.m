function out = b_string_utils(inputText, sep)
%B_STRING_UTILS Normalise, split and summarise a block of text.
%   OUT = B_STRING_UTILS(TEXT) returns a struct with cleaned words, a
%   keyword count and a short summary line.
%
%   OUT = B_STRING_UTILS(TEXT, SEP) splits on SEP instead of spaces.

if nargin < 2
    sep = ' ';
end

% Accept string arrays by joining them into one char row
if isstring(inputText)
    inputText = char(join(inputText, newline));
end

txt = strtrim(inputText);
txt = regexprep(txt, '\s+', ' ');
txt = lower(txt);

% Split, drop empties and strip punctuation
words = strsplit(txt, sep);
words = words(~cellfun(@isempty, words));
words = regexprep(words, '[^a-z0-9'']', '');
words = words(~cellfun(@isempty, words));

% Remove stop words
keep = ~ismember(words, stopwords());
out.words  = words(keep);
out.nWords = numel(out.words);
out.label  = string(out.nWords) + " words";

% Word frequencies in a containers.Map
counts = containers.Map('KeyType', 'char', 'ValueType', 'double');
for i = 1:numel(out.words)
    w = out.words{i};
    if isKey(counts, w)
        counts(w) = counts(w) + 1;
    else
        counts(w) = 1;
    end
end
out.counts = counts;

% Top five words by frequency
ks   = counts.keys;
vals = cell2mat(counts.values);
[~, order] = sort(vals, 'descend');
top = ks(order(1:min(5, numel(order))));

out.summary = sprintf('%d unique words, top: %s', counts.Count, ...
    strjoin(top, ', '));

% Word lengths as a comma-separated list built with sprintf
lens = cellfun(@length, out.words);
out.lengthList = sprintf('%d,', lens);
if ~isempty(out.lengthList)
    out.lengthList(end) = [];
end

% Simple flags with contains and startsWith
out.hasQuestion      = contains(txt, '?');
out.startsWithHello  = startsWith(txt, 'hello');

% Short title as a string, and the first word as char
out.title = string(txt(1:min(40, numel(txt))));
out.firstWord = char(extractBefore([string(txt) " "], " "));

% Lookup in a cell array of char
labels = {'alpha', 'beta', 'gamma'};
idx = find(strcmp(labels, 'beta'));
out.betaIndex = idx;

% Number formatting: num2str keeps the precision of the input
out.ratio = num2str(out.nWords / max(numel(words), 1), 3);

end

function sw = stopwords()
%STOPWORDS Common English stop words used by the word filter.
sw = {'the', 'a', 'an', 'of', 'and', 'to', 'in', 'is', 'it'};
end
