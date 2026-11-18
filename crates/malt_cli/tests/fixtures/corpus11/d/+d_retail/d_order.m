classdef d_order
    % d_order is a customer order with lines of {sku, qty, unitCents}.

    properties
        Id (1,1) double
        Lines cell = {}
        Status (1,:) char = 'open'
    end

    methods
        function obj = d_order(id)
            if nargin == 1
                obj.Id = id;
            end
        end

        function addLine(obj, sku, qty, unitCents)
            obj.Lines{end+1} = struct('sku', sku, 'qty', qty, 'unit', unitCents);
        end

        function total = totalCents(obj)
            total = 0;
            for k = 1:numel(obj.Lines)
                line = obj.Lines{k};
                total = total + line.qty * line.unit;
            end
        end

        function close(obj)
            if isempty(obj.Lines)
                error('d_retail:d_order:empty', 'd_order %d has no lines', obj.Id);
            end
            obj.Status = 'closed';
        end
    end

    methods (Static)
        function o = fromRows(rows)
            o = d_retail.d_order(rows{1, 1});
            for r = 2:size(rows, 1)
                o.addLine(rows{r, 1}, rows{r, 2}, rows{r, 3});
            end
            o.close();
        end
    end
end
