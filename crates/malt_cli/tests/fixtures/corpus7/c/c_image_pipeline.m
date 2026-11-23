% R3_IMAGE_PIPELINE  Load an image, threshold it, count blobs and save a mask.
% Expects an input image named input.png in the working folder.

img = imread(fullfile(pwd, 'input.png'));
if size(img, 3) == 3
    gray = rgb2gray(img);
else
    gray = img;
end
gray = im2double(gray);

level = graythresh(gray);
mask = imbinarize(gray, level);
mask = imfill(mask, 'holes');
mask = bwareaopen(mask, 50);

cc = bwconncomp(mask);
stats = regionprops(cc, 'Area', 'Centroid', 'BoundingBox');
areas = [stats.Area]
nBlobs = numel(stats)

big = stats([stats.Area] > 500);
fprintf('%d blobs over 500 px\n', numel(big));

centroids = zeros(numel(big), 2);
for k = 1:numel(big)
    centroids(k, :) = big(k).Centroid;
end

labels = labelmatrix(cc);
colored = label2rgb(labels, 'jet', 'k');

figure
imshow(colored)
title(sprintf('%d blobs', nBlobs))

imwrite(mask, 'mask.png');
clear img gray
histCounts = histcounts(double(labels(:)), 0:nBlobs);

h = imhist(im2uint8(gray), 16);
[peakCount, peakBin] = max(h);
peakLevel = (peakBin - 1) / 15

if nBlobs == 0
    disp('nothing found')
end

mm = [min(areas) max(areas)];
ratio = mm(2) / mm(1);
resized = imresize(gray, 0.5);
gx = imgradient(resized);
edges = edge(resized, 'canny');
nEdges = nnz(edges);
