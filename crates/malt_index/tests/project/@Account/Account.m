classdef Account < handle
    % A bank account.
    properties
        Balance = 0
    end
    methods
        function obj = Account(opening)
            obj.Balance = opening;
        end
    end
end
