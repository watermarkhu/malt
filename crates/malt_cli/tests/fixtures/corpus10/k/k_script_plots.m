%% Plot library exercises: a script that builds a few figures for a memo
% Writes PNG files into a temp folder; run it by hand.

outDir = tempname;
mkdir(outDir)
t = linspace(0, 2*pi, 200);
y1 = sin(t);
y2 = cos(t);

f1 = figure('Visible', 'off');
ax = axes(f1);
plot(ax, t, y1, 'b', t, y2, 'r')
grid on
title('Sine and cosine')
saveas(f1, fullfile(outDir, 'trig.png'))
close(f1)

f2 = figure('Visible', 'off');
histogram(y1 + 0.1*randn(size(y1)), 20)
exportgraphics(f2, fullfile(outDir, 'hist.png'))
close all

for k = 1:3
    fk = figure('Visible', 'off');
    surf(peaks(k*10))
    shading interp
    colorbar
    saveas(fk, sprintf('%s/peaks%d.png', outDir, k))
end

result = dir(fullfile(outDir, '*.png'));
names = {result.name};
fprintf('wrote %d files\n', numel(names))
delete(fullfile(outDir, '*.png'))
rmdir(outDir)
x = 0:0.1:1;
z = x.^2;
plot(x, z)
