from collections import defaultdict
from datetime import datetime

from django.contrib.auth import get_user_model
from django.db.models import Count
from django.views.generic.list import ListView
from django.views.generic.detail import DetailView


class PilotList(ListView):
    model = get_user_model()
    template_name = 'stats/pilot_list.html'


class PilotDetail(DetailView):
    model = get_user_model()
    template_name = 'stats/pilot_detail.html'

    def get_context_data(self, **kwargs):
        context = super().get_context_data(**kwargs)

        # Flight count
        context['flight_count'] = self.object.flight_set.count()
        context['flight_count_per_year'] = self.object.flight_set \
                .values_list('launch_date__year') \
                .annotate(Count('id')) \
                .order_by('launch_date__year')

        # Flight duration
        timings = self.object.flight_set \
                .values_list('launch_date', 'launch_time', 'landing_time') \
                .filter(launch_time__isnull=False, landing_time__isnull=False) \
                .order_by('launch_date', 'launch_time')
        durations = [
            {
                'year': t[0].year,
                'seconds': (datetime.combine(t[0], t[2]) - datetime.combine(t[0], t[1])).seconds,
            }
            for t in timings
        ]
        context['flight_duration'] = round(sum(d['seconds'] for d in durations) / 3600, 2)
        durations_per_year = defaultdict(int)
        for d in durations:
            durations_per_year[d['year']] += d['seconds']
        context['flight_duration_per_year'] = {
            year: round(seconds / 3600, 2)
            for year, seconds in durations_per_year.items()
        }

        # Get location stats
        context['flight_launches'] = \
            len(set(self.object.flight_set.values_list('launch_at', flat=True)))
        context['flight_landings'] = \
            len(set(self.object.flight_set.values_list('landing_at', flat=True)))
        context['flight_countries'] = \
            sorted(set(self.object.flight_set.values_list('landing_at__country', flat=True)))

        # Get flying years
        context['years'] = \
            sorted(set(self.object.flight_set.values_list('launch_date__year', flat=True)))

        return context
