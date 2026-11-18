classdef d_account_handle < handle
%D_ACCOUNT_HANDLE Bank account with a transaction log and a low-balance event.
%   A handle class: copies of an account refer to the same balance.
%
%   acc = d_account_handle('Alice', 500);
%   acc.deposit(200);
%   acc.withdraw(1000);     % throws d_account_handle:overdraft
%   acc.Transactions        % struct array of the log

    events
        LowBalance
    end

    properties (SetAccess = private)
        Owner (1,1) string
        Balance (1,1) double = 0
        Transactions struct = struct('type', {}, 'amount', {}, 'balanceAfter', {})
    end

    properties (Access = private)
        LowThreshold (1,1) double = 100
        IsClosed (1,1) logical = false
    end

    methods
        function obj = d_account_handle(owner, openingBalance)
            % Constructor; the opening balance is optional
            obj.Owner = string(owner);
            if nargin > 1
                obj.Balance = openingBalance;
            end
            obj.logTransaction("open", obj.Balance);
        end

        function deposit(obj, amount)
            obj.assertOpen();
            obj.Balance = obj.Balance + amount;
            obj.logTransaction("deposit", amount);
        end

        function withdraw(obj, amount)
            obj.assertOpen();
            if amount > obj.Balance
                error('d_account_handle:overdraft', ...
                    'Withdrawal of %.2f exceeds balance %.2f.', amount, obj.Balance);
            end
            obj.Balance = obj.Balance - amount;
            obj.logTransaction("withdraw", -amount);

            % Let listeners know when the account gets low
            if obj.Balance < obj.LowThreshold
                notify(obj, 'LowBalance');
            end
        end

        function close(obj)
            obj.IsClosed = true;
        end

        function delete(obj)
            % Warn if the account object goes away while still open
            if ~obj.IsClosed
                fprintf('Account for %s deleted while open (balance %.2f).\n', ...
                    obj.Owner, obj.Balance);
            end
        end
    end

    methods (Access = private)
        function assertOpen(obj)
            if obj.IsClosed
                error('d_account_handle:closed', 'Account is closed.');
            end
        end

        function logTransaction(obj, kind, amount)
            n = numel(obj.Transactions);
            obj.Transactions(n + 1).type = kind;
            obj.Transactions(n + 1).amount = amount;
            obj.Transactions(n + 1).balanceAfter = obj.Balance;
        end
    end
end
