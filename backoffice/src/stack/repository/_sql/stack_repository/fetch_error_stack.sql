select id, error_name, error_summary, error_stack, reported_at
from error_stack
where id = :id