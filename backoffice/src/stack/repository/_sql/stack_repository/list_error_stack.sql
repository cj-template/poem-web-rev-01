select id, error_name, error_summary, reported_at
from error_stack
where reported_at > datetime('now', '-30 day')
order by id desc