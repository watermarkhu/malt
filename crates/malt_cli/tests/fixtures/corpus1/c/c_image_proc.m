function [mask, stats] = c_image_proc(img, thresh, kernelSize)
%C_IMAGE_PROC Threshold a grayscale image and label connected blobs.
%   [MASK, STATS] = C_IMAGE_PROC(IMG, THRESH, KERNELSIZE) smooths IMG with
%   a box filter, thresholds it and labels the foreground components.

if nargin < 3
    kernelSize = 5;
end
if nargin < 2 || isempty(thresh)
    thresh = 0.5;
end

% Work in double precision for the convolution
A = double(img);
A = A ./ max(A(:));

% Box filter kernel
K = ones(kernelSize) / kernelSize^2;
smoothed = conv2(A, K, 'same');

% Threshold to a logical mask
mask = smoothed > thresh;
mask = mask & ~isnan(smoothed);

% Hand-written component labelling (4-connected)
labels = zeros(size(mask));
nextLabel = 0;
[nRows, nCols] = size(mask);
for r = 1:nRows
    for c = 1:nCols
        if ~mask(r, c) || labels(r, c) ~= 0
            continue
        end
        nextLabel = nextLabel + 1;
        queue = [r, c];
        labels(r, c) = nextLabel;
        while ~isempty(queue)
            p = queue(1, :);
            queue(1, :) = [];
            nb = [p(1)-1 p(2); p(1)+1 p(2); p(1) p(2)-1; p(1) p(2)+1];
            for k = 1:size(nb, 1)
                rr = nb(k, 1);
                cc = nb(k, 2);
                if rr < 1 || cc < 1 || rr > nRows || cc > nCols
                    continue
                end
                if mask(rr, cc) && labels(rr, cc) == 0
                    labels(rr, cc) = nextLabel;
                    queue(end+1, :) = [rr, cc]; %#ok<AGROW>
                end
            end
        end
    end
end

% Per-component statistics
stats = struct('label', {}, 'area', {}, 'meanIntensity', {});
for L = 1:nextLabel
    inComp = labels == L;
    stats(L).label = L;
    stats(L).area = nnz(inComp);
    stats(L).meanIntensity = mean(A(inComp));
end

% Keep only components with a reasonable area
minArea = 20;
keep = [stats.area] >= minArea;
stats = stats(keep);

% Mean and spread of the whole image, for the log
globalMean = mean(A(:));
globalStd = std(A(:));
fprintf('mean=%.3f std=%.3f components=%d\n', globalMean, globalStd, numel(stats));

% Per-channel normalisation of a stack of tiles
tiles = reshape(1:(nRows*nCols), nRows, nCols);
tiles = permute(tiles, [2 1]);
tileMeans = arrayfun(@(k) mean(tiles(:, k)), 1:size(tiles, 2));
tileMeans = tileMeans(:);

% Preallocate the output buffer the same size as the mask
out = zeros(size(mask));
out(mask) = smoothed(mask);
sharpened = A + 0.5 * (A - conv2(A, K, 'same'));
tmp = sharpened(:, end:-1:1);

end
