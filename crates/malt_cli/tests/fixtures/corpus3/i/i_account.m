classdef i_account < handle
    % I_ACCOUNT  a simple bank account
    %   acc = i_account('Bob', 100);
    %   acc.deposit(20);
    %   acc.withdraw(500);   % throws i_account:funds

    properties
        Owner = 'nobody'
        Balance = 0
        Overdraft = 0
        History = {}
    end

    properties (Constant)
        Currency = 'EUR'
    end

    methods
        function obj = i_account(owner, opening)
            % build an account; both arguments are optional
            if nargin >= 1
                obj.Owner = owner;
            end
            if nargin >= 2
                obj.Balance = opening;
            end
        end

        function deposit(obj, amount)
            % add money to the account
            if amount <= 0
                error('i_account:amount', 'deposit must be positive');
            end
            obj.Balance = obj.Balance + amount;
            obj.History{end+1} = ['deposit ' num2str(amount)];
        end

        function withdraw(obj, amount)
            % take money out; the overdraft limit is respected
            if amount > obj.Balance + obj.Overdraft
                error('i_account:funds', 'insufficient funds: %g < %g', ...
                    obj.Balance + obj.Overdraft, amount);
            end
            obj.Balance = obj.Balance - amount;
            obj.History{end+1} = ['withdraw ' num2str(amount)];
        end

        function transfer(obj, other, amount)
            % move money to another account object
            obj.withdraw(amount);
            other.deposit(amount);
        end

        function s = summary(obj)
            % one line description
            s = sprintf('%s: %.2f %s (%d ops)', obj.Owner, obj.Balance, ...
                obj.Currency, numel(obj.History));
        end
    end
end
