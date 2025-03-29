create table if not exists color (
    color_id    serial not null,
    hexadecimal char(6) not null check (hexadecimal  ~ '^[0-9A-F]{6}$'),
    
    constraint pk_fruit_color primary key (color_id)
);

create table if not exists locale (
    locale_id char(5),
    lang      varchar(64),
    
    constraint pk_locale_code primary key (locale_id)
);

insert into locale (locale_id, lang) values
    ('en_US', 'English (US)'),
    ('de_DE', 'Deutsch');

create table if not exists text_label (
    label_id varchar(255) not null unique,

    constraint pk_text_label primary key (label_id)
);

insert into text_label (label_id) values 
    ('FRUIT_COLOR'),
    ('FRUIT_DESCRIPTION');

create table if not exists localized_text_content (
    localized_text_content_id serial not null,
    locale_id                 char(5) not null,
    label                     varchar(255) not null,
    option_reference_id       int,
    value                     text not null,
    
    constraint unique_locale_label unique(locale_id, label, option_reference_id),
    constraint fk_text_content_label foreign key (label) references text_label(label_id)
);

create index label_index on localized_text_content(label);

create table if not exists fruit (
    fruit_id             serial not null,
    fruit_name           varchar(128) not null,
    color                int not null,
    avg_weight_in_grams  decimal(5, 2) not null,
    scoville_range_start int not null,
    scoville_range_end   int,
    
    constraint pk_fruit primary key (fruit_id),
    constraint fk_fruit_color foreign key (color) references color(color_id)
);

create table if not exists plant (
    plant_id         char(11) not null,
    planted          date not null,
    disposed         date,
    fruit            int not null,
    notes            varchar(255),
    is_label_printed bool not null default false,
    
    constraint pk_plant primary key (plant_id),
    constraint fk_plant_fruit foreign key (fruit) references fruit(fruit_id)
);

create or replace function fn_set_plant_id()
returns trigger as $$
declare
    part_fruit_name char(3);
    part_year_month char(4);
    last_id         char(4);
    incremented     char(4);
begin
    select upper(substring(fruit.fruit_name from 1 for 3)) into part_fruit_name from fruit
    where fruit.fruit_id = new.fruit;
    
    part_year_month := to_char(new.planted, 'YYMM');
    
    select substring(plant_id from 8 for 4) into last_id from plant
    where plant.plant_id like part_fruit_name || part_year_month || '%'
    order by plant.plant_id desc
    limit 1;
    
    if last_id is not null then
    	incremented := cast(last_id as int) + 1;
    else 
    	incremented := 1;
    end if;
    
    new.plant_id = part_fruit_name || part_year_month || lpad(cast(incremented as text), 4, '0');
    
    return new;
end;
$$ language plpgsql;

create trigger trg_set_plant_id
before insert on plant
for each row
execute function fn_set_plant_id();

create table if not exists harvest (
    harvest_id   serial not null,
    harvest_date date not null default current_timestamp,
    notes        text,
    
    constraint pk_harvest primary key (harvest_id)
);

create table if not exists harvest_plant (
    harvest_fruit_id serial not null,
    harvest          int not null,
    plant            char(11) not null,
    weight_in_grams  decimal(5, 2) not null,
    
    constraint pk_harvest_fruit primary key (harvest_fruit_id),
    constraint fk_harvest foreign key (harvest) references harvest(harvest_id),
    constraint fk_plant foreign key (plant) references plant(plant_id)
);
