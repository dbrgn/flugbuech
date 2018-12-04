from rest_framework import viewsets

from data import models as data_models
from . import serializers


class FlightViewSet(viewsets.ModelViewSet):
    """
    API endpoint that allows flights to be viewed or edited.
    """
    serializer_class = serializers.FlightSerializer
    queryset = data_models.Flight.objects.all()
