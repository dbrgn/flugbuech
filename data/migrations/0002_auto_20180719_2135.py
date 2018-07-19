# Generated by Django 2.1rc1 on 2018-07-19 21:35

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('data', '0001_initial'),
    ]

    operations = [
        migrations.AlterField(
            model_name='aircraft',
            name='brand',
            field=models.CharField(blank=True, help_text='e.g. Skywalk', max_length=100),
        ),
        migrations.AlterField(
            model_name='aircraft',
            name='name',
            field=models.CharField(help_text='e.g. Tequila 4', max_length=255),
        ),
    ]
