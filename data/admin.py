from django.contrib import admin

from . import models


@admin.register(models.Aircraft)
class AircraftAdmin(admin.ModelAdmin):
    pass


@admin.register(models.Location)
class LocationAdmin(admin.ModelAdmin):
    list_display = ('name', 'country', 'altitude')


@admin.register(models.Flight)
class FlightAdmin(admin.ModelAdmin):
    list_display = ('pk', 'pilot', 'aircraft', 'number', 'launch_date', 'launch_time', 'launch_at')
    list_filter = ('pilot', 'launch_at', 'landing_at')
    date_hierarchy = 'launch_date'
    save_as = True
