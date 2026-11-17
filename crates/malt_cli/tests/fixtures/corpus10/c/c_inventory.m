classdef c_inventory < handle
    % c_inventory tracks stock levels and announces low stock through events.

    events
        LowStock
        Restocked
    end

    properties
        Threshold (1,1) double {mustBeNonnegative} = 5
    end

    properties (SetAccess = protected)
        Items struct = struct('sku', {}, 'qty', {})
    end

    properties (Access = private)
        Log cell = {}
    end

    methods
        function obj = c_inventory(threshold)
            if nargin > 0
                obj.Threshold = threshold;
            end
        end

        function add(obj, sku, qty)
            arguments
                obj
                sku (1,:) char
                qty (1,1) double {mustBeInteger}
            end
            idx = find(strcmp({obj.Items.sku}, sku), 1);
            if isempty(idx)
                obj.Items(end+1) = struct('sku', sku, 'qty', qty);
            else
                obj.Items(idx).qty = obj.Items(idx).qty + qty;
            end
            obj.Log{end+1} = sprintf('add %s %d', sku, qty);
            notify(obj, 'Restocked');
        end

        function take(obj, sku, qty)
            idx = find(strcmp({obj.Items.sku}, sku), 1);
            if isempty(idx)
                error('c_inventory:unknownSku', 'No item %s', sku);
            end
            obj.Items(idx).qty = obj.Items(idx).qty - qty; %#ok<*NASGU>
            if obj.Items(idx).qty < obj.Threshold
                evt = struct('sku', sku, 'qty', obj.Items(idx).qty);
                notify(obj, 'LowStock', evt)
            end
        end

        function n = count(obj)
            n = numel(obj.Items);
        end
    end

    methods (Static)
        function inv = fromFile(fname)
            txt = fileread(fname);
            lines = splitlines(txt);
            inv = c_inventory();
            for k = 1:numel(lines)
                parts = split(lines(k), ',');
                if numel(parts) == 2
                    inv.add(strtrim(parts(1)), str2double(parts(2)));
                end
            end
        end
    end
end
