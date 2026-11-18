%% Build a binary mask from a synthetic image and measure its regions
img = zeros(64, 80);
[cols, rows] = meshgrid(1:80, 1:64);
img((rows - 20).^2 + (cols - 30).^2 < 100) = 1;
img((rows - 40).^2 + (cols - 60).^2 < 144) = 1;
img = img + 0.05 * randn(size(img));

mask = img > 0.5;
mask = imfill(mask, 'holes');
cc = bwconncomp(mask);
stats = regionprops(cc, 'Area', 'Centroid');
areas = [stats.Area];
big = find(areas > 50);
fprintf('%d regions, %d large\n', cc.NumObjects, numel(big));

for k = 1:numel(big)
    c = stats(big(k)).Centroid;
    fprintf('region %d centroid (%.1f, %.1f)\n', k, c(1), c(2));
end

edges = edge(double(mask), 'canny');
nEdge = nnz(edges)
kernel = ones(3) / 9;
smooth = conv2(double(mask), kernel, 'same');
thresholded = smooth > 0.5;
labels = bwlabel(thresholded);
numLabels = max(labels(:))
sizes = accumarray(labels(labels > 0), 1);
largest = max(sizes);
meanIntensity = mean(img(mask))
s = std(img(:));
if s < 0.1
    disp('low contrast')
end
