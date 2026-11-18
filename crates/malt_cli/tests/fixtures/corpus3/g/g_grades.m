clear all; close all; clc
% CS101 grades, section 2 (fall)
% quick and dirty, names typed in by hand from the roster

names = {'Ann', 'Bob', 'Carlos', 'Dee', 'Eli', 'Fay', 'Gus'}
scores = [88 72 95; 65 58 70; 91 85 79; 45 52 60; 78 80 74; 99 97 100; 58 61 49];
weights = [0.3 0.3 0.4];
n = size(scores, 1);
avg = zeros(n, 1);
for i = 1:n
    avg(i) = sum(scores(i,:) .* weights);
end
avg

% grade letters
grade = cell(n, 1);
for i = 1:n
    if avg(i) >= 90
        grade{i} = 'A';
    elseif avg(i) >= 80
        grade{i} = 'B';
    elseif avg(i) >= 70
        grade{i} = 'C';
    elseif avg(i) >= 60
        grade{i} = 'D';
    else
        grade{i} = 'F';
    end
end

% struct array
students = struct('name', {}, 'avg', {}, 'grade', {});
for i = 1:n
    students(i).name = names{i};
    students(i).avg = avg(i);
    students(i).grade = grade{i};
end

% bubble sort by average, descending
sorted = avg;
sortedNames = names;
for i = 1:n-1
    for j = 1:n-i
        if sorted(j) < sorted(j+1)
            tmp = sorted(j); sorted(j) = sorted(j+1); sorted(j+1) = tmp;
            tn = sortedNames{j}; sortedNames{j} = sortedNames{j+1}; sortedNames{j+1} = tn;
        end
    end
end

fprintf('Rank  Name      Avg\n')
for i = 1:n
    fprintf('%4d  %-8s  %6.2f\n', i, sortedNames{i}, sorted(i));
end

% count letters with switch on strings
counts = struct('A', 0, 'B', 0, 'C', 0, 'D', 0, 'F', 0);
for i = 1:n
    switch grade{i}
        case 'A'
            counts.A = counts.A + 1;
        case 'B'
            counts.B = counts.B + 1;
        case 'C'
            counts.C = counts.C + 1;
        case 'D'
            counts.D = counts.D + 1;
        case 'F'
            counts.F = counts.F + 1;
    end
end
counts

% same thing with strcmp, written later by someone else
nA = 0; nF = 0;
for i = 1:n
    if strcmp(grade{i}, 'A')
        nA = nA + 1;
    end
    if strcmp(grade{i}, 'F') == 1
        nF = nF + 1;
    end
end
disp(['A grades: ' num2str(nA) ', F grades: ' num2str(nF)])

% special case checks
for i = 1:n
    if strcmp(names{i}, 'Gus') == 1
        disp(['Gus is in row ' num2str(i) ', grade ' grade{i}])
    end
end
if names{1} == 'Ann'
    disp('first is Ann')
end
if avg(1) > 90 | avg(2) > 90
    disp('someone did well')
end

% dump for the department, old format
fid = fopen('grades_out.txt', 'w');
for i = 1:n
    fprintf(fid, '%s;%.1f;%s\n', names{i}, avg(i), grade{i});
end
fclose(fid);

% class average, printed without semicolon
classAvg = mean(avg)
disp(['Class average: ' num2str(classAvg)])
fprintf('%s\n', 'done')

% grade letter by number of points, duplicated block
for i = 1:n
    if avg(i) >= 90
        lt{i} = 'A';
    elseif avg(i) >= 80
        lt{i} = 'B';
    else
        lt{i} = 'other';
    end
end

% unused variables left over
nothing = 1;
tmp2 = 'hello';
maxScore = max(max(scores))
% minScore = min(min(scores));

% number of students with each grade via find
idxA = find(strcmp(grade, 'A'));
if isempty(idxA)
    disp('no As this term')
else
    fprintf('As: %s\n', mat2str(idxA'))
end

% 'histogram' by hand
bins = zeros(1, 5);
for i = 1:n
    b = floor(avg(i)/20) + 1;
    if b > 5, b = 5; end
    bins(b) = bins(b) + 1;
end
bins
figure
bar(bins)
xlabel('bin')
ylabel('count')
