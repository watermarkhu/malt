% pendulum + spring analysis, spring 2017 data
% ran this on the lab PC, paths are hard-coded, sorry
clear all; close all; clc

%% constants
g = 9.81
L = 0.75;      % m, string length (measured with ruler)
m = 0.2;       % kg
k_spring = 14.2;
files = {'run1.csv', 'run2.csv', 'run3.csv', 'run4.csv', 'run5.csv'};
nfiles = length(files);
results = zeros(1, nfiles);
allT = {};
names = {};

%% loop over csv files and estimate period by zero crossings
for i = 1:length(files)
    fname = files{i};
    if ~exist(fname, 'file')
        disp(['missing file ' fname])
        continue
    end
    data = csvread(fname, 1, 0);
    t = data(:,1);
    th = data(:,2);
    n = length(t);
    zc = [];
    for k = 2:n
        if th(k-1) > 0 & th(k) <= 0
            zc = [zc t(k)];
        end
    end
    if length(zc) < 2
        fprintf('run %d: not enough crossings\n', i)
        T = NaN;
    else
        T = mean(diff(zc))*2;
    end
    results(i) = T;
    allT{i} = t;
    names{end+1} = fname;
end

%% period from the theory, for comparison
T_theory = 2*pi*sqrt(L/g)
T_theory_damped = T_theory * 1.0

%% second pass: amplitude decay
amp = [];
amp_end = [];
for j = 1:nfiles
    if j > length(allT) || isempty(allT{j})
        continue;
    end
    data = csvread(files{j}, 1, 0);
    th = data(:,2);
    pk = [];
    for q = 2:length(th)-1
        if th(q) > th(q-1) && th(q) > th(q+1) && th(q) > 0
            pk(end+1) = th(q);
        end
    end
    if ~isempty(pk)
        amp(j) = pk(1);
        amp_end(j) = pk(end);
    end
end

%% damping estimate: log decrement
decay = zeros(1, nfiles);
for j = 1:length(amp)
    if amp(j) == 0 | isnan(amp(j))
        decay(j) = NaN;
    else
        decay(j) = log(amp(j)/amp_end(j)) / (length(allT{j})-1);
    end
end
decay

%% plots
figure(1)
hold on
for j = 1:nfiles
    if isempty(allT{j}), continue, end
    plot(allT{j}, results(j)*ones(size(allT{j})), '--')
end
hold off
xlabel('time (s)')
ylabel('period')
title('periods per run')
legend(names)

figure(2)
plot(1:nfiles, results, 'o-')
hold on
plot(1:nfiles, T_theory*ones(1,nfiles), 'r--')
hold off
xlabel('run')
ylabel('T (s)')

% same plot again, copied from above, changed the colour
figure(3)
plot(1:nfiles, results, 'o-')
hold on
plot(1:nfiles, T_theory*ones(1,nfiles), 'k--')
hold off
xlabel('run')
ylabel('T (s)')

%% summary stats
valid = results(~isnan(results));
mu = mean(valid)
sd = std(valid);
fprintf('mean period %.4f s, sd %.4f s\n', mu, sd)
err = abs(mu - T_theory)/T_theory * 100
if err > 5 | sd > 0.2
    disp('check the setup')
end

%% old way, from before we had csvread
% fid = fopen('run1.txt');
% C = textscan(fid, '%f %f', 'HeaderLines', 1);
% fclose(fid);
% T1 = mean(C{1});

%% load the spring data from .mat files
for q = 1:3
    eval(['mat_' num2str(q) ' = load(''mat' num2str(q) '.mat'');'])
end
d = load('data_final.mat');
vals = d.vals;
i = 3;
kk = vals(i)
unused_thing = 42;

%% energy per run
E = zeros(1, nfiles);
for i = 1:nfiles
    if isnan(results(i))
        E(i) = NaN;
    else
        omega = 2*pi/results(i);
        E(i) = 0.5*k_spring*(0.1)^2 + 0.5*m*(L*omega*0.1)^2;
    end
end
E
Emax = max(E)
Emin = min(E(~isnan(E)))

%% spring constant from the static stretch data
stretch = [0.01 0.02 0.03 0.04 0.05];
force = [0.14 0.29 0.44 0.58 0.73];
p = polyfit(stretch, force, 1);
k_fit = p(1)
if abs(k_fit - k_spring) > 1
    fprintf('k differs: fit %.2f, nominal %.2f\n', k_fit, k_spring)
end

%% output table
fid = fopen('results_summary.txt', 'w');
fprintf(fid, 'run,period,energy\n');
for i = 1:nfiles
    fprintf(fid, '%d,%f,%f\n', i, results(i), E(i));
end
fclose(fid);

%% per-run plots of raw data, one figure per file
for i = 1:nfiles
    if ~exist(files{i}, 'file')
        continue
    end
    dd = csvread(files{i}, 1, 0);
    figure
    plot(dd(:,1), dd(:,2))
    title(files{i})
    xlabel('t')
    ylabel('theta')
    drawnow
end

%% ratio of periods, sloppy
for i = 1:nfiles-1
    for j = i+1:nfiles
        r(i,j) = results(i)/results(j);
    end
end
r

%% grow arrays in a loop (fine for 5 runs)
history = [];
for i = 1:nfiles
    history = [history; results(i) E(i)];
end
size(history)

%% quick check whether the ruler measurement is ok
if L > 1 | L < 0.1
    error('L looks wrong: %f', L)
end
disp('done')
