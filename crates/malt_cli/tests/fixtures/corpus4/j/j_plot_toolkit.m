function h = j_plot_toolkit(x, y, varargin)
%J_PLOT_TOOLKIT  Handle graphics helpers for the lab report figures.
%   H = J_PLOT_TOOLKIT(X, Y) plots Y against X in figure 1 with the house
%   style.  Extra options are passed as name/value pairs:
%     'title', 'xlabel', 'ylabel', 'color', 'nofig'
%
%   Written for MATLAB 5.3 and kept working on later releases.

opt.title = 'untitled';
opt.xlabel = 'x';
opt.ylabel = 'y';
opt.color = 'b';
opt.nofig = 0;
for k = 1:2:length(varargin)
    opt = setfield(opt, varargin{k}, varargin{k+1});
end

if ~opt.nofig
    figure(1)
    clf
    set(gcf, 'Color', 'w')
    set(gcf, 'PaperPosition', [0.25 2.5 8 6])
end

h = line(x, y, 'Color', opt.color, 'LineWidth', 1.5);
set(gca, 'FontSize', 10, 'Box', 'on', 'TickDir', 'out')
title(opt.title)
xlabel(opt.xlabel)
ylabel(opt.ylabel)
grid on

% make the figure a little prettier, every time, in the old way
j_style_axes(gca);

% a second axes beside the first, using the handle
hax = axes('Position', [0.6 0.6 0.3 0.3]);
axes(hax)
plot(x, y, 'k.')
set(hax, 'FontSize', 8)
axes(get(gcf, 'CurrentAxes'))

% hold toggles, as in the original scripts
if ishold
    disp('hold was already on')
end
hold on
plot(x, y + 0.1 * std(y), 'g:')
hold off

% some labels placed by hand with gtext, only when interactive
if opt.nofig == 0 & usejava('desktop')
    gtext('peak');
end

% annotate the maximum
[ymax, imax] = max(y);
text(x(imax), ymax, ' max', 'FontSize', 9, 'VerticalAlignment', 'bottom')

% legend, built from the list of names in the global list
global PLOT_LEGENDS
if ~isempty(PLOT_LEGENDS)
    legend(PLOT_LEGENDS)
end

% colour map for the surface figure, if one is drawn
cmap = colormap(jet(64));
colormap(cmap)
caxis([min(y) max(y)])

h = [h j_add_stamp(gcf)];
end_text = get(h(1), 'Parent');
set(end_text, 'Visible', 'on')
drawnow
end

function out = j_add_stamp(fh)
% put a date stamp in the lower left corner of the figure
out = text(0.02, 0.02, ['made ' date], 'Units', 'normalized', ...
           'FontSize', 7, 'Color', [0.4 0.4 0.4]);
set(fh, 'UserData', out);
end

function j_style_axes(ax)
% the house style for the axes, applied to any axes handle
set(ax, 'Box', 'on')
set(ax, 'LineWidth', 1)
set(ax, 'XColor', [0 0 0], 'YColor', [0 0 0])
ticks = get(ax, 'XTick');
if length(ticks) > 8
    set(ax, 'XTick', ticks(1:2:end))
end
end

function j_export(fh, fname)
% print the figure as encapsulated postscript and as a pdf
figure(fh)
print -depsc2 fname
print -dpdf fname
saveas(fh, [fname '.fig'])
disp(['exported ' fname])
end

function j_surface_demo
% a small demo of the surface helpers, not called from the report
[X, Y] = meshgrid(-2:0.1:2, -2:0.1:2);
Z = X .* exp(-X.^2 - Y.^2);
figure(2)
surf(X, Y, Z)
shading flat
colorbar
view(-37.5, 30)
xlabel('X'), ylabel('Y'), zlabel('Z')
end
