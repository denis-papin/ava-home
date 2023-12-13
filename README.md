"# ava-home" 

Control your Zigbee home devices

Si nous avions un service par device  
    * 
    *

interType1 -- topic "inter_dim_kitchen"
interType1 -- topic "inter_dim_room"
lamp1 -- topic "lamp_kitchen"

Le pattern1 = (interType1, interType1, )

On capte un changement d'état 
[ 
    
]

La notion de boucle est essentielle mais définir des boucles pour chacun des cas est très long 

on pourrait imaginer un service par pattern, réutilisable avec des "devices" différents ne partageant pas forcement 
la meme forme de message 

il serait nécessaire d'avoir un framework pour gérer facile le codage d'un pattern 

loop1 : patternLoop --topics = [.../.../...]
loop2 : patternLop --topics = [.../.../...]

Exemple de boucle : 
   
    appuie sur inter1 
    inter1 envoie un message sur mqtt (set) via z2m et qui signale son état
    l'écoute de son topic d'état signale un changement d'état
    fabrication du message pour la lampe1 -> envoi 
    la lampe 1 recoit son message de changement d'état (set)
    fabrication du message pour la lampe2 -> envoi 
    la lampe 2 recoit son message de changement d'état (set)
    
    via le dashboard, lamp1 change d'état
    son nouvel état est capté par la boucle 
    fabrication du message pour l'inter1 -> envoi
    fabrication du message pour la lampe2 -> envoi


Generics 
Enums
Config files
No locks