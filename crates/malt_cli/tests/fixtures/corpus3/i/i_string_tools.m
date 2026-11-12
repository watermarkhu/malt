function res = i_string_tools(word)
% I_STRING_TOOLS  string algorithms in char-array style
%   res = i_string_tools(word)

if nargin < 1
    word = 'Racecar';
end

res.is_pal = is_palindrome(word);
res.caesar = caesar(word, 3);
res.uncaesar = caesar(res.caesar, -3);
res.anagram = is_anagram('listen', 'silent');
res.counts = count_words('the cat and the hat and the bat');
res.nums = parse_numbers('1 2 3 42');
res.rev = fliplr(word);
res.joined = join_with_comma({'a', 'bb', 'ccc'});
res.matched = regexp_demo('abc123def45');
fprintf('%s is palindrome: %d\n', word, res.is_pal);
fprintf('caesar: %s\n', res.caesar);
end

function tf = is_palindrome(s)
% two pointers moving towards the middle
s = lower(s);
tf = true;
i = 1;
j = length(s);
while i < j
    if s(i) ~= s(j)
        tf = false;
        return;
    end
    i = i + 1;
    j = j - 1;
end
end

function out = caesar(s, k)
% shift letters by k, keep everything else as it is
out = '';
for i = 1:length(s)
    c = s(i);
    if c >= 'a' && c <= 'z'
        out = [out char(mod(double(c) - 97 + k, 26) + 97)];
    elseif c >= 'A' && c <= 'Z'
        out = [out char(mod(double(c) - 65 + k, 26) + 65)];
    else
        out = [out c];
    end
end
end

function tf = is_anagram(a, b)
% same letters after sorting
a = sort(lower(a(a ~= ' ')));
b = sort(lower(b(b ~= ' ')));
tf = strcmp(a, b);
end

function m = count_words(sentence)
% word frequencies in a containers.Map
words = strsplit(sentence, ' ');
m = containers.Map();
for i = 1:numel(words)
    w = words{i};
    if isKey(m, w)
        m(w) = m(w) + 1;
    else
        m(w) = 1;
    end
end
end

function v = parse_numbers(s)
% split a string and convert each piece with str2num
parts = strsplit(s, ' ');
v = [];
for i = 1:length(parts)
    v(end+1) = str2num(parts{i}); %#ok<ST2NM>
end
end

function s = join_with_comma(c)
% join a cell array of strings with commas
s = '';
for i = 1:length(c)
    if i > 1
        s = strcat(s, ', ');
    end
    s = strcat(s, c{i});
end
end

function out = regexp_demo(s)
% pull out digit runs and letter runs
digits = regexp(s, '\d+', 'match');
letters = regexp(s, '[a-z]+', 'match');
out = [strjoin(digits, '|') ' / ' strjoin(letters, '|')];
out = [out ' ' upper(num2str(numel(digits)))];
end
