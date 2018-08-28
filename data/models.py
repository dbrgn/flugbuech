from django.contrib.auth import get_user_model
from django.db import models

from django_countries.fields import CountryField


class Aircraft(models.Model):
    pilot = models.ForeignKey(get_user_model(), null=False, on_delete=models.CASCADE)
    name = models.CharField(max_length=255,
            help_text='e.g. Tequila 4')
    brand = models.CharField(max_length=100,
            help_text='e.g. Skywalk', blank=True)

    def __str__(self):
        return '{} {}'.format(self.brand, self.name)


class Location(models.Model):
    name = models.CharField(max_length=255)
    country = CountryField()
    altitude = models.IntegerField(
            help_text='Elevation above sea level (AMSL) in meters')

    def __str__(self):
        return '{} ({})'.format(self.name, self.country)


class Flight(models.Model):
    pilot = models.ForeignKey(get_user_model(), null=False, on_delete=models.CASCADE)
    aircraft = models.ForeignKey(Aircraft, null=True, blank=True, on_delete=models.SET_NULL)
    number = models.IntegerField(null=True, blank=True,
            help_text='The flight number')
    launch_at = models.ForeignKey(Location, null=False, blank=False, on_delete=models.PROTECT,
            related_name='flight_launch')
    launch_date = models.DateField()
    launch_time = models.TimeField(null=True, blank=True)
    landing_at = models.ForeignKey(Location, null=True, blank=True, on_delete=models.PROTECT,
            related_name='flight_landing')
    landing_time = models.TimeField(null=True, blank=True)
    max_altitude = models.IntegerField(null=True, blank=True,
            help_text='The max altitude reached during the flight')
    track_distance = models.IntegerField(null=True, blank=True,
            help_text='The track distance in kilometers')
    comments = models.TextField(blank=True)
    video = models.URLField(null=True, blank=True,
            help_text='A link to a video of your flight')

    class Meta:
        unique_together = [
            ('pilot', 'number'),
        ]

    def __str__(self):
        return 'Flight {}'.format(self.pk)
