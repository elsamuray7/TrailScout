import L from "leaflet";
import { Sight } from "../data/Sight";

export const sightsIcon = L.icon({
  iconUrl: 'assets/icons/sights.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const nightIcon = L.icon({
  iconUrl: 'assets/icons/nachtleben.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const restaurantIcon = L.icon({
  iconUrl: 'assets/icons/restaurant.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const shoppingIcon = L.icon({
  iconUrl: 'assets/icons/shopping.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const grillIcon = L.icon({
  iconUrl: 'assets/icons/grill.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const museumIcon = L.icon({
  iconUrl: 'assets/icons/museum.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const natureIcon = L.icon({
  iconUrl: 'assets/icons/natur.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const seaIcon = L.icon({
  iconUrl: 'assets/icons/see.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const activitiesIcon = L.icon({
  iconUrl: 'assets/icons/activities.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const animalsIcon = L.icon({
  iconUrl: 'assets/icons/animals.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

export const startIcon = L.icon({
  iconUrl: 'assets/icons/start.png',

  iconSize: [30, 35], // size of the icon
  // shadowSize:   [50, 64], // size of the shadow
  // iconAnchor:   [22, 94], // point of the icon which will correspond to marker's location
  // shadowAnchor: [4, 62],  // the same for the shadow
  // popupAnchor:  [-3, -76] // point from which the popup should open relative to the iconAnchor
});

const iconRetinaUrl = 'assets/marker-icon-2x.png';
const iconUrl = 'assets/marker-icon.png';
const shadowUrl = 'assets/marker-shadow.png';
export const iconDefault = L.icon({
  iconRetinaUrl,
  iconUrl,
  shadowUrl,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  tooltipAnchor: [16, -28],
  shadowSize: [41, 41]
});

export function getIcon(sight: Sight) {
  const cat = sight.category;
  switch (cat) {
    case "Sightseeing":
      return sightsIcon;
    case "Nightlife":
      return nightIcon;
    case "Restaurants":
      return restaurantIcon;
    case "Shopping":
      return shoppingIcon;
    case "PicnicBarbequeSpot":
      return grillIcon;
    case "MuseumExhibition":
      return museumIcon;
    case "Nature":
      return natureIcon;
    case "Swimming":
      return seaIcon;
    case "Activities":
      return activitiesIcon;
    case "Animals":
      return animalsIcon;
    default:
      return iconDefault;
  }
}

