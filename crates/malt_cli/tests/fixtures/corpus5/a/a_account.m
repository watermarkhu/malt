classdef a_account < handle
    % R1_ACCOUNT  A simple bank account with a transaction log.
    %   Used by the round 1 probe. Amounts are in cents.

    properties (SetAccess = private)
        Balance (1,1) double {mustBeNonnegative} = 0
        Owner (1,:) char
        History cell = {}
    end

    properties (Constant)
        MaxOverdraft = -500
    end

    properties (Access = private)
        lastId = 0;
    end

    events
        BalanceChanged
        Overdrawn
    end

    methods
        function obj = a_account(owner, opening)
            if nargin < 2
                opening = 0
            end
            obj.Owner = owner;
            obj.Balance = opening;
            obj.History{end+1} = sprintf('open %d', opening);
        end

        function deposit(obj, amount)
            if amount <= 0
                error('r1:account:badAmount', 'Amount must be positive.');
            end
            obj.Balance = obj.Balance + amount;
            obj.log('deposit', amount);
            notify(obj, 'BalanceChanged');
        end

        function withdraw(obj, amount)
            newBalance = obj.Balance - amount;
            if newBalance < obj.MaxOverdraft
                notify(obj, 'Overdrawn');
                error('r1:account:overdraft', 'Limit reached');
            end
            obj.Balance = newBalance;
            obj.log('withdraw', amount);
            notify(obj, 'BalanceChanged')
        end

        function n = numTransactions(obj)
            n = numel(obj.History);
        end
    end

    methods (Access = private)
        function log(obj, kind, amount)
            obj.lastId = obj.lastId + 1;
            entry = struct('id', obj.lastId, 'kind', kind, 'amount', amount);
            obj.History{end+1} = entry;
        end
    end

    methods (Static)
        function c = fromCents(x)
            c = x / 100;
        end
    end
end
