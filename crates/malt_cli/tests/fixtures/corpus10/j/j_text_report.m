function report = text_report(records)
%TEXT_REPORT Formats a fixed-width text report from a cell array of records.
%   Each record is {name, score, comment}. Long comments wrap at 40 columns.

%{
Layout: name padded to 12 characters, score right aligned in 6 columns,
then the wrapped comment. The header is written once.
%}

report = '';
header = sprintf('%-12s%6s  %s', 'Name', 'Score', 'Comment');
report = [report header newline];
report = [report repmat('-', 1, numel(header)) newline];

for k = 1:numel(records)
    rec = records{k};
    name = rec{1};
    score = rec{2};
    comment = rec{3};
    if isnumeric(score) && isscalar(score)
        scoreText = sprintf('%6.1f', score);
    else
        scoreText = sprintf('%6s', 'n/a');
    end
    wrapped = wrap(comment, 40);
    report = [report sprintf('%-12s%s  %s', name, scoreText, wrapped{1}) newline];
    for j = 2:numel(wrapped)
        report = [report sprintf('%-12s%6s  %s', '', '', wrapped{j}) newline];
    end
end
end

function lines = wrap(txt, width)
words = strsplit(txt, ' ');
lines = {};
current = '';
for w = 1:numel(words)
    candidate = strtrim([current ' ' words{w}]);
    if length(candidate) <= width
        current = candidate;
    else
        lines{end+1} = current; %#ok<AGROW>
        current = words{w};
    end
end
lines{end+1} = current;
end
