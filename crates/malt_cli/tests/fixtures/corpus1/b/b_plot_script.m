%% Plot a family of decay curves for several datasets
% Produces one figure per dataset plus a summary figure and a heat map.
clear
clc
close all
rng(42)

datasets = {'run_a', 'run_b', 'run_c'};
nRuns    = numel(datasets);
t        = linspace(0, 10, 200);
rates    = [0.3, 0.6, 0.9];
colors   = lines(nRuns);

outDir = fullfile(tempdir, 'b_plot_out');
if ~exist(outDir, 'dir')
    mkdir(outDir);
end

stats = zeros(nRuns, 2);

for k = 1:nRuns
    % Noisy exponential decay for this dataset
    model = exp(-rates(k) * t);
    y = model + 0.02 * randn(size(t));
    stats(k, :) = [mean(y), std(y)];

    fig = figure('Name', datasets{k}, 'Color', 'w');

    % Top panel: data against the model
    subplot(2, 1, 1)
    plot(t, y, '.', 'Color', colors(k, :))
    hold on
    plot(t, model, 'k--', 'LineWidth', 1.5)
    hold off
    xlabel('time [s]')
    ylabel('signal')
    title(sprintf('%s (rate = %.2f)', datasets{k}, rates(k)))
    legend('data', 'model', 'Location', 'northeast')
    axis tight

    % Bottom panel: residual on a log scale
    subplot(2, 1, 2)
    semilogy(t, abs(y - model) + eps, '-o', 'MarkerSize', 2)
    xlabel('time [s]')
    ylabel('|residual|')
    grid on

    drawnow
    saveas(fig, fullfile(outDir, sprintf('%s.png', datasets{k})))
end

% Summary bar chart with error bars
figure('Name', 'summary', 'Color', 'w')
bar(categorical(datasets), stats(:, 1))
hold on
errorbar(1:nRuns, stats(:, 1), stats(:, 2), 'k.', 'LineStyle', 'none')
hold off
ylabel('mean signal')
title('Mean and spread per dataset')
ax = gca;
ax.YLim = [0, 1.2];

% Heat map of the model over time and rate
rateGrid = linspace(0.1, 1, 30);
surfaceVals = exp(-rateGrid' * t);
figure('Name', 'heat', 'Color', 'w')
imagesc(t, rateGrid, surfaceVals)
colormap(hot)
c = colorbar;
c.Label.String = 'signal';
xlabel('time [s]')
ylabel('rate')
axis xy

fprintf('Saved %d figures to %s\n', nRuns + 1, outDir);
