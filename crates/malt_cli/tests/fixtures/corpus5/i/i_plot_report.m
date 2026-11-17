% R1_PLOT_REPORT  Plot the weekly temperature table and save it.
% Figures are created off-screen in the saving step.

%{
Layout: two panels, a daily line and a weekly bar chart.
The data is generated, not loaded, so the script runs anywhere.
%}

days = 1:7;
temps = [14.2 15.1 13.8 16.0 17.4 18.1 16.9];
rain  = [0 2.4 0 0 5.1 0.2 0];

fig = figure('Name', 'Weekly report', 'Visible', 'off');
subplot(2, 1, 1)
plot(days, temps, '-o', 'LineWidth', 1.5);
grid on
xlabel('Day'); ylabel('Temperature (C)')
title(sprintf('Mean %.1f C', mean(temps)))

subplot(2, 1, 2)
bar(days, rain)
ax = gca;
ax.XTick = days;
ax.XTickLabel = {'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'};

total_rain = sum(rain) ...
    + 0;   % continued statement
legend('Rain', 'Location', 'northwest')

outFile = fullfile(tempdir, 'weekly_report.png');
saveas(fig, outFile)
close(fig)

wetDays = find(rain > 0);
fprintf('wet days: %s\n', mat2str(wetDays))
dryFraction = 1 - numel(wetDays) / numel(days)

hi = max(temps);
[~, hiDay] = max(temps);
text_out = ['Hottest day was ' num2str(hiDay) ...
            ' at ' num2str(hi) ' C'];
disp(text_out)

color = [0.2 0.4 0.6];
set(gca, 'Color', color);
hold on
hold off
