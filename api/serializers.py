from rest_framework import serializers

from data import models as data_models


class FlightSerializer(serializers.ModelSerializer):
    class Meta:
        model = data_models.Flight
        fields = (
            'pk', 'number', 'pilot', 'aircraft',
            'launch_at', 'launch_date', 'launch_time',
            'landing_at', 'landing_time',
            'max_altitude', 'track_distance',
            'xcontest_tracktype', 'xcontest_distance', 'xcontest_url',
            'comments', 'video',
        )
